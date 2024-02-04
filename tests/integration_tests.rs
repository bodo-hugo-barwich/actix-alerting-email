#[cfg(test)]
mod tests {
    use actix::sync::SyncArbiter;
    use actix_web::{http::header::ContentType, test, web, App};

    use alerting_email::config::AppConfig;
    use alerting_email::email::{EmailData, EmailLink, EmailResponse, EmailSender};
    use alerting_email::{dispatch_home_page, dispatch_ping_request, send_email, ResponseData};

    #[actix_rt::test]
    async fn test_home() {
        let mut app =
            test::init_service(App::new().route("/", web::get().to(dispatch_home_page))).await;
        let req = test::TestRequest::with_header("content-type", ContentType::json()).to_request();

        let resp = test::call_service(&mut app, req).await;

        println!("home hdrs: '{:?}'", resp);

        assert!(resp.status().is_success());

        let response: ResponseData = test::read_body_json(resp).await;

        println!("send bdy: '{:?}'", response);

        assert_eq!(response.page.as_str(), "Home");
        assert_eq!(response.statuscode, 200);
    }

    #[actix_rt::test]
    async fn test_ping() {
        let mut app =
            test::init_service(App::new().route("/ping", web::get().to(dispatch_ping_request)))
                .await;
        let req = test::TestRequest::with_header("content-type", ContentType::json())
            .uri("/ping")
            .to_request();

        let resp = test::call_service(&mut app, req).await;

        println!("ping hdrs: '{:?}'", resp);

        assert!(resp.status().is_success());

        let response: ResponseData = test::read_body_json(resp).await;

        println!("send bdy: '{:?}'", response);

        assert_eq!(response.page.as_str(), "Ping");
        assert_eq!(response.statuscode, 200);
    }

    #[actix_rt::test]
    async fn test_send() {
        let config = AppConfig::from_file();
        //Clone the SMTP Config for the Email Worker
        let smtp_config = config.smtp.clone();

        println!("app config: {:?}", config);

        //Create 2 Email Sender Instances
        let sender = SyncArbiter::start(1, move || EmailSender::from_config(&smtp_config));
        //Create 1 Email Link Object
        let link = EmailLink::new(sender);

        let mut app = test::init_service(
            App::new()
                .app_data(web::Data::new(link.clone()))
                .route("/send", web::post().to(send_email)),
        )
        .await;

        let email = EmailData {
            subject: String::from("[Mail Test - ")
                + config.component.as_str()
                + "] my test subject",
            from: String::from("sender@testmail.com"),
            to: String::from("receiver@testmail.com"),
            message: String::from("Mail Test - ")
                + config.component.as_str()
                + "\n===============\n\nmy test email message",
        };
        let req = test::TestRequest::post()
            .uri("/send")
            .set_json(&email)
            .to_request();
        let resp = test::call_service(&mut app, req).await;

        println!("send hdrs: '{:?}'", resp);

        assert!(resp.status().is_success());

        let response: EmailResponse = test::read_body_json(resp).await;

        println!("send bdy: '{:?}'", response);

        assert_eq!(response.status.as_str(), "sent");
    }
}
