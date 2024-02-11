#![allow(dead_code)]

use core::fmt::Debug;
use std::{any::Any, fmt::Display, marker::PhantomData};

// trait DisplayWithDebugTrait: Display + Debug {}

trait PinService<'a, Request> {
    type Response: Display;

    fn call(&mut self, req: Request) -> Self::Response;
}

struct UntypedService<'a, S, Request, Response>
where
    S: PinService<'a, Request, Response = Response>,
    S::Response: 'a,
    Request: Any,
{
    svc: S,
    _phantom: PhantomData<&'a fn(Request) -> Response>,
}

impl<'a, S, Request, Response> PinService<'a, Box<dyn Any>>
    for UntypedService<'a, S, Request, Response>
where
    S: PinService<'a, Request, Response = Response>,
    S::Response: 'a,
    Request: Any,
    Response: 'a + ToString + Display,
{
    type Response = Box<dyn 'a + Display>;

    fn call(&mut self, req: Box<dyn Any>) -> Self::Response {
        let req: Request = *req.downcast::<Request>().ok().expect("wrong request type");

        let resp = self.svc.call(req);
        Box::new(resp.to_string())
    }
}

type BoxUntypedService<'a> =
    Box<dyn 'a + PinService<'a, Box<dyn Any>, Response = Box<dyn 'a + Display>>>;

fn boxed_untyped_service<'a, S, Request>(svc: S) -> BoxUntypedService<'a>
where
    S: 'a + PinService<'a, Request, Response = Box<(dyn std::fmt::Display + 'a)>>,
    S::Response: 'a,
    Request: Any,
{
    Box::new(UntypedService {
        svc,
        _phantom: PhantomData,
    })
}

// #[derive(Default)]
// struct ServiceMap {
//     map: HashMap<TypeId, BoxUntypedService<'static>>,
// }

// impl ServiceMap {
//     fn register<S, Request>(&mut self, svc: S)
//     where
//         S: 'static + PinService<Request>,
//         S::Response: 'static,
//         S::Response: 'static + Debug,
//         Request: Any,
//     {
//         self.map
//             .insert(TypeId::of::<Request>(), boxed_untyped_service(svc));
//     }
// }

// impl ServiceMap {
//     async fn call<Request>(&mut self, req: Request)
//     where
//         Request: Any,
//     {
//         let Some(svc) = self.map.get_mut(&TypeId::of::<Request>()) else {
//             return;
//         };

//         let resp = svc.call(Box::new(req)).await;
//         println!("{resp:?}");
//     }
// }

#[derive(Debug)]
struct Foo {}

impl<'a> PinService<'a, i64> for Foo {
    type Response = Box<dyn 'a + Display>;

    fn call(&mut self, req: i64) -> Self::Response {
        Box::new(format!("got i64: {req:?}"))
    }
}

#[derive(Debug)]
struct Point(f64, f64);

struct Bar {}

impl<'a> PinService<'a, (f64, f64)> for Bar {
    type Response = Box<dyn 'a + Display>;

    fn call(&mut self, req: (f64, f64)) -> Self::Response {
        Box::new(format!("{:?}", Point(req.0, req.1)))
    }
}

fn main() {
    let mut svc = boxed_untyped_service(Foo {});

    let resp = svc.call(Box::new(123i64));
    println!("{resp}");

    let resp = boxed_untyped_service(Bar {}).call(Box::new((123.0f64, 456.0f64)));
    println!("{resp}");
}
