#[cfg(all(test, feature = "mockdata"))]
pub mod tests {
    use actix_web::http::StatusCode;
    use chrono::{DateTime, Duration, Utc};
    use serde_json;
    use vrb_common::api_error::code_to_str;
    use vrb_dbase::db_enums::UserRole;
    use vrb_tools::err;

    use crate::profiles::{profile_check::uniqueness_nickname_or_email, profile_models::Profile, profile_orm::tests::ProfileOrmApp};
    use crate::users::{user_models::UserRegistr, user_registr_orm::tests::UserRegistrOrmApp};

    fn get_data() -> (Vec<Profile>, Vec<UserRegistr>) {
        let nickname = "Oliver_Taylor";
        let email = format!("{}@gmail.com", nickname);
        let role = UserRole::User;
        let profile = ProfileOrmApp::new_profile(1, nickname, &email, role);

        let now = Utc::now();
        let final_date: DateTime<Utc> = now + Duration::minutes(20);
        let nickname = "Robert_Brown";
        let email = format!("{}@gmail.com", nickname);
        let password = "passwdR2B2";
        let user_registr = UserRegistrOrmApp::new_user_registr(1, nickname, &email, password, final_date);

        (vec![profile], vec![user_registr])
    }

    fn get_orm(data: (Vec<Profile>, Vec<UserRegistr>)) -> (ProfileOrmApp, UserRegistrOrmApp) {
        let profile_orm = ProfileOrmApp::create2(&data.0);
        let user_registr_orm = UserRegistrOrmApp::create(&data.1);

        (profile_orm, user_registr_orm)
    }

    // ** uniqueness_nickname_or_email **

    #[actix_web::test]
    async fn test_uniqueness_nickname_or_email_by_non_params() {
        let data = get_data();
        let opt_nickname: Option<String> = None;
        let opt_email: Option<String> = None;
        let (profile_orm, user_registr_orm) = get_orm(data);

        let result = uniqueness_nickname_or_email(opt_nickname, opt_email, profile_orm, user_registr_orm).await;
        assert!(result.is_err());
        let app_err = result.err().unwrap();
        assert_eq!(app_err.code, code_to_str(StatusCode::NOT_ACCEPTABLE));
        assert_eq!(app_err.message, err::MSG_PARAMS_NOT_SPECIFIED);
        #[rustfmt::skip]
        let json = serde_json::json!({ "nickname": "null", "email": "null" });
        assert_eq!(*app_err.params.get("invalidParams").unwrap(), json);
    }
    #[actix_web::test]
    async fn test_uniqueness_nickname_or_email_by_nickname_empty() {
        let data = get_data();
        let opt_nickname: Option<String> = Some("".to_string());
        let opt_email: Option<String> = None;
        let (profile_orm, user_registr_orm) = get_orm(data);

        let result = uniqueness_nickname_or_email(opt_nickname, opt_email, profile_orm, user_registr_orm).await;
        assert!(result.is_err());
        let app_err = result.err().unwrap();
        assert_eq!(app_err.code, code_to_str(StatusCode::NOT_ACCEPTABLE));
        assert_eq!(app_err.message, err::MSG_PARAMS_NOT_SPECIFIED);
        #[rustfmt::skip]
        let json = serde_json::json!({ "nickname": "null", "email": "null" });
        assert_eq!(*app_err.params.get("invalidParams").unwrap(), json);
    }
    #[actix_web::test]
    async fn test_uniqueness_nickname_or_email_by_email_empty() {
        let data = get_data();
        let opt_nickname: Option<String> = None;
        let opt_email: Option<String> = Some("".to_string());
        let (profile_orm, user_registr_orm) = get_orm(data);

        let result = uniqueness_nickname_or_email(opt_nickname, opt_email, profile_orm, user_registr_orm).await;
        assert!(result.is_err());
        let app_err = result.err().unwrap();
        assert_eq!(app_err.code, code_to_str(StatusCode::NOT_ACCEPTABLE));
        assert_eq!(app_err.message, err::MSG_PARAMS_NOT_SPECIFIED);
        #[rustfmt::skip]
        let json = serde_json::json!({ "nickname": "null", "email": "null" });
        assert_eq!(*app_err.params.get("invalidParams").unwrap(), json);
    }
    #[actix_web::test]
    async fn test_uniqueness_nickname_or_email_by_nickname_profile() {
        let data = get_data();
        let opt_nickname: Option<String> = Some(data.0.get(0).unwrap().nickname.clone());
        let opt_email: Option<String> = None;
        let (profile_orm, user_registr_orm) = get_orm(data);

        let result = uniqueness_nickname_or_email(opt_nickname, opt_email, profile_orm, user_registr_orm).await;
        assert!(result.is_ok());
        assert_eq!(result.ok().unwrap(), Some((true, false)));
    }
    #[actix_web::test]
    async fn test_uniqueness_nickname_or_email_by_email_profile() {
        let data = get_data();
        let opt_nickname: Option<String> = None;
        let opt_email: Option<String> = Some(data.0.get(0).unwrap().email.clone());
        let (profile_orm, user_registr_orm) = get_orm(data);

        let result = uniqueness_nickname_or_email(opt_nickname, opt_email, profile_orm, user_registr_orm).await;
        assert!(result.is_ok());
        assert_eq!(result.ok().unwrap(), Some((false, true)));
    }
    #[actix_web::test]
    async fn test_uniqueness_nickname_or_email_by_nickname_registr() {
        let data = get_data();
        let nickname_registr = data.1.get(0).unwrap().nickname.clone();
        let (profile_orm, user_registr_orm) = get_orm(data);

        let opt_nickname: Option<String> = Some(nickname_registr);
        let opt_email: Option<String> = None;

        let result = uniqueness_nickname_or_email(opt_nickname, opt_email, profile_orm, user_registr_orm).await;
        assert!(result.is_ok());
        assert_eq!(result.ok().unwrap(), Some((true, false)));
    }
    #[actix_web::test]
    async fn test_uniqueness_nickname_or_email_by_email_registr() {
        let data = get_data();
        let email_registr = data.1.get(0).unwrap().email.clone();
        let (profile_orm, user_registr_orm) = get_orm(data);

        let opt_nickname: Option<String> = None;
        let opt_email: Option<String> = Some(email_registr);

        let result = uniqueness_nickname_or_email(opt_nickname, opt_email, profile_orm, user_registr_orm).await;
        assert!(result.is_ok());
        assert_eq!(result.ok().unwrap(), Some((false, true)));
    }
    #[actix_web::test]
    async fn test_uniqueness_nickname_or_email_by_new_nickname() {
        let data = get_data();
        let nickname1 = format!("a{}", data.0.get(0).unwrap().nickname.clone());
        let (profile_orm, user_registr_orm) = get_orm(data);

        let opt_nickname: Option<String> = Some(nickname1);
        let opt_email: Option<String> = None;

        let result = uniqueness_nickname_or_email(opt_nickname, opt_email, profile_orm, user_registr_orm).await;
        assert!(result.is_ok());
        assert_eq!(result.ok().unwrap(), None);
    }
    #[actix_web::test]
    async fn test_uniqueness_nickname_or_email_by_new_email() {
        let data = get_data();
        let email1 = format!("a{}", data.0.get(0).unwrap().email.clone());
        let (profile_orm, user_registr_orm) = get_orm(data);

        let opt_nickname: Option<String> = None;
        let opt_email: Option<String> = Some(email1);

        let result = uniqueness_nickname_or_email(opt_nickname, opt_email, profile_orm, user_registr_orm).await;
        assert!(result.is_ok());
        assert_eq!(result.ok().unwrap(), None);
    }
}
