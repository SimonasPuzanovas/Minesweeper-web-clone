use crate::responses::{resp_give_uuid, resp_log_reg_status};
use actix_web::{HttpRequest, HttpResponse};
use bson::{doc, Document};
use mongodb::Collection;
use uuid::Uuid;

#[derive(Clone)]
pub struct Services {
    pub users: Collection<Document>,
}

impl Services {
    pub fn new(usr: Collection<Document>) -> Self {
        Self { users: usr }
    }

    //USER SERVICE

    pub async fn handle_cookie(&self, req: &HttpRequest) -> HttpResponse {
        match req.cookie("uuid") {
            Some(cookie) => {
                match self
                    .users
                    .find_one(doc! {"uuid": cookie.value()}, None)
                    .await
                    .unwrap()
                {
                    None => HttpResponse::Ok().body(std::fs::read("login.html").unwrap()),
                    Some(_user) => HttpResponse::Ok().body(std::fs::read("game.html").unwrap()),
                }
            }
            None => HttpResponse::Ok().body(std::fs::read("login.html").unwrap()),
        }
    }

    pub async fn add_user(&self, name: &String, password: &String) -> HttpResponse {
        let legalchars = "qwertyuiopasdfghjklzxcvbnmQWERTYUIOPLKJHGFDSAZXCVBNM123456789";
        println!("hey");
        if 4<=name.len() && 4<=password.len() && name.len()<= 10 && password.len()<= 10{
            for c in name.chars(){
                let mut legal = false;
                for cc in legalchars.chars(){
                    if c == cc {legal = true;}
                }
                if !legal {return resp_log_reg_status("usage of special symbols is not allowed");}
            }
            for c in password.chars(){
                let mut legal = false;
                for cc in legalchars.chars(){
                    if c == cc {legal = true;}
                }
                if !legal {return resp_log_reg_status("usage of special symbols is not allowed");}
            }
        }
        else {return resp_log_reg_status("username or password might be too short or too long");}

        match self.users.find_one(doc! {"name": name}, None).await {
            Ok(result) => match result {
                None => {
                    self.users
                        .insert_one(
                            doc! {
                                "name": name,
                                "password": password,
                                "uuid": "none",
                            },
                            None,
                        )
                        .await
                        .unwrap();
                    resp_log_reg_status("You're successfully created an account.")
                }
                Some(_) => resp_log_reg_status("Username is taken."),
            },
            Err(_) => resp_log_reg_status("Internal register server error."),
        }
    }

    pub async fn login_user(&self, name: &String, password: &String) -> HttpResponse {
        let uuid = Uuid::new_v4().simple().to_string();

        match self
            .users
            .update_one(
                doc! {"name": name, "password": password},
                doc! {"$set": {"uuid": &uuid}},
                None,
            )
            .await
        {
            Ok(result) => {
                if result.modified_count == 0 {
                    resp_log_reg_status("Login failed.")
                } else {
                    resp_give_uuid(uuid)
                }
            }
            Err(_) => resp_log_reg_status("Internal login server error."),
        }
    }
}
