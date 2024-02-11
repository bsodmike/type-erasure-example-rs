#![allow(dead_code)]

use core::fmt::Debug;

use std::{
    any::{Any, TypeId},
    collections::HashMap,
    fs::read_to_string,
    marker::PhantomData,
    pin::Pin,
};

trait PinService<Request> {
    type Response;
    type ReturnType: ToString;

    fn call(&mut self, req: Request) -> Self::ReturnType;
}

struct UntypedService<'a, S, Request, Response>
where
    S: PinService<Request, Response = Response>,
    S::ReturnType: 'a,
    Request: Any,
    Response: Debug,
{
    svc: S,
    _phantom: PhantomData<&'a fn(Request) -> Response>,
}

impl<'a, S, Request, Response> PinService<Box<dyn Any>> for UntypedService<'a, S, Request, Response>
where
    S: PinService<Request, Response = Response>,
    S::ReturnType: 'a,
    Request: Any,
    Response: 'a + Debug,
{
    type Response = Box<dyn 'a + Debug>;
    type ReturnType = String;

    fn call(&mut self, req: Box<dyn Any>) -> Self::ReturnType {
        let req: Request = *req.downcast::<Request>().ok().expect("wrong request type");

        let resp = self.svc.call(req);
        resp.to_string()
    }
}

type BoxUntypedService<'a> =
    Box<dyn 'a + PinService<Box<dyn Any>, Response = Box<dyn 'a + Debug>, ReturnType = String>>;

fn boxed_untyped_service<'a, S, Request>(svc: S) -> BoxUntypedService<'a>
where
    S: 'a + PinService<Request>,
    S::ReturnType: 'a,
    S::Response: 'a + Debug,
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
//         S::ReturnType: 'static,
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

fn main() {
    let mut svc = boxed_untyped_service(Foo {});

    let resp = svc.call(Box::new(123i64));
    println!("{resp:?}");

    let resp = boxed_untyped_service(Bar {}).call(Box::new((123.0f64, 456.0f64)));
    println!("{resp:?}");
}

struct Foo {}

impl PinService<i64> for Foo {
    type Response = String;
    type ReturnType = String;

    fn call(&mut self, req: i64) -> Self::ReturnType {
        // ready(format!("got i64: {req:?}"))
        format!("got i64: {req:?}")
    }
}

#[derive(Debug)]
struct Point(f64, f64);

struct Bar {}

impl PinService<(f64, f64)> for Bar {
    type Response = Point;
    type ReturnType = String;

    fn call(&mut self, req: (f64, f64)) -> Self::ReturnType {
        format!("{:?}", Point(req.0, req.1))
    }
}
