use super::*;
use crate::DestinationPolicy;
use crate::NetworkPolicyController;
use futures::future::poll_fn;
use http_body::Body;
use http_body::Frame;
use pretty_assertions::assert_eq;
use std::collections::VecDeque;

struct Frames(VecDeque<Frame<Bytes>>);

impl Body for Frames {
    type Data = Bytes;
    type Error = reqwest::Error;

    fn poll_frame(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Bytes>, Self::Error>>> {
        Poll::Ready(self.0.pop_front().map(Ok))
    }
}

#[tokio::test]
async fn framed_body_preserves_trailers_and_stops_after_revocation() {
    let controller = NetworkPolicyController::default();
    let policy = controller.policy();
    controller.publish(policy.revision(), DestinationPolicy::Unrestricted);
    let permit = policy
        .acquire(&reqwest::Url::parse("https://gateway.example/").unwrap())
        .unwrap();
    let mut trailers = http::HeaderMap::new();
    trailers.insert("grpc-status", http::HeaderValue::from_static("0"));
    let revoked = permit.clone();
    let mut body = PolicyBody {
        body: Some(Box::pin(Frames(VecDeque::from([
            Frame::data(Bytes::from_static(b"response")),
            Frame::trailers(trailers.clone()),
        ])))),
        permit,
        revoked: async move { revoked.revoked().await }.boxed(),
    };
    let data = poll_fn(|cx| Pin::new(&mut body).poll_frame(cx))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(data.into_data().unwrap(), Bytes::from_static(b"response"));
    let frame = poll_fn(|cx| Pin::new(&mut body).poll_frame(cx))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(frame.into_trailers().unwrap(), trailers);
    body.body = Some(Box::pin(Frames(VecDeque::from([Frame::data(
        Bytes::from_static(b"must not be delivered after revocation"),
    )]))));
    policy.invalidate();
    let failure = poll_fn(|cx| Pin::new(&mut body).poll_frame(cx))
        .await
        .unwrap()
        .unwrap_err();
    assert!(matches!(
        failure,
        HttpError::Policy(crate::NetworkPolicyDenied::Revoked)
    ));
    assert!(
        poll_fn(|cx| Pin::new(&mut body).poll_frame(cx))
            .await
            .is_none()
    );
}
