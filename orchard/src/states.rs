use axum::extract::FromRef;

use crate::services::{
    authentication_service::AuthenticationService, country_service::CountryService,
    notification_service::NotificationService, root_service::RootService,
    user_service::UserService, wait_list_service::WaitListService,
};

use async_graphql::dynamic::Schema;
use seaography::async_graphql;

#[derive(Clone)]
pub struct ServicesState {
    pub user_service: UserService,
    pub root_service: RootService,
    pub auth_service: AuthenticationService,
    pub notification_service: NotificationService,
    pub country_service: CountryService,
    pub wait_list_service: WaitListService,
}

impl FromRef<ServicesState> for UserService {
    fn from_ref(services: &ServicesState) -> UserService {
        services.user_service.clone()
    }
}

impl FromRef<ServicesState> for RootService {
    fn from_ref(services: &ServicesState) -> RootService {
        services.root_service.clone()
    }
}

impl FromRef<ServicesState> for AuthenticationService {
    fn from_ref(services: &ServicesState) -> AuthenticationService {
        services.auth_service.clone()
    }
}

impl FromRef<ServicesState> for CountryService {
    fn from_ref(input: &ServicesState) -> CountryService {
        input.country_service.clone()
    }
}

impl FromRef<ServicesState> for NotificationService {
    fn from_ref(services: &ServicesState) -> NotificationService {
        services.notification_service.clone()
    }
}

impl FromRef<ServicesState> for WaitListService {
    fn from_ref(services: &ServicesState) -> WaitListService {
        services.wait_list_service.clone()
    }
}


#[derive(Clone)]
pub struct GraphQlState {
    pub schema: Schema,
    pub endpoint: String,
}
