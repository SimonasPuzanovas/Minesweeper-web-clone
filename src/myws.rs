use actix::prelude::*;
use actix_web_actors::ws;
//use serde::{Deserialize, Serialize};
use crate::games;
use crate::games::Games;
use std::time::{Duration, Instant};

const HB_INTERVAL: Duration = Duration::from_secs(5);
const HB_TIMEOUT: Duration = Duration::from_secs(10);

pub struct MyWs{
    pub heartbeat: Instant,
    pub addr: Addr<Games>,
    pub gameid: String,
    pub username: String,
}

impl MyWs{
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>){
        ctx.run_interval(HB_INTERVAL, |act, ctx|{
            if act.heartbeat.elapsed() > HB_TIMEOUT{
                //If in game too then remove
                act.addr.do_send(games::Offline{
                    username: act.username.clone(),
                    addr: ctx.address().clone(),
                });
                act.addr.do_send(games::DisconnectFromGame{
                    username: act.username.clone(),
                    addr: ctx.address().clone(),
                    gameid: act.gameid.clone(),
                });

                ctx.stop();
                return
            }
            ctx.text("");
        });
    }
}

impl Actor for MyWs {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context){
        self.hb(ctx);
        self.addr.do_send(games::Online{
            username: self.username.clone(), addr: ctx.address().clone()})
    }
}

impl Handler<games::ServerMsg> for MyWs{
    type Result = ();

    fn handle(&mut self, msg: games::ServerMsg, ctx: &mut Self::Context){
        ctx.text(msg.data);
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MyWs {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>,
                                        ctx: &mut Self::Context) {

        match msg {
            Ok(ws::Message::Text(text)) => {
                if text == "" {self.heartbeat = Instant::now()}
                //player wants gamestate
                else if text == "/gamestate"{
                    self.addr.do_send(games::GameState{
                        addr: ctx.address().clone(),
                        username: self.username.clone(),
                    });
                }
                else if text == "/resetgame"{
                    self.addr.do_send(games::ResetGame{
                        gameid: self.gameid.clone(),
                    });
                }

                else if text == "/changecolor"{
                    self.addr.do_send(games::ChangeColor{
                        username: self.username.clone(),
                        gameid: self.gameid.clone()
                    })
                }

                else{
                    //split strings
                    let vector: Vec<&str> = text.split(" ").collect();

                    //player connects to game room
                    if vector[0] == "/joinroom"{
                        self.addr.do_send(games::ConnectToGame{
                            gameid: vector[1].to_string(),
                            addr: ctx.address().clone(),
                            username: self.username.clone(),
                        });
                        self.gameid = vector[1].to_string();
                    }

                    //player disconnects
                    else if vector[0] == "/quitroom"{
                        self.addr.do_send(games::DisconnectFromGame{
                            username: self.username.clone(),
                            addr: ctx.address().clone(),
                            gameid: self.gameid.clone(),
                        });
                    }

                    //if moderator then kick specific user
                    else if vector[0] == "/kick"{
                        self.addr.do_send(games::Kick{
                            gameid: self.gameid.clone(),
                            username: self.username.clone(),
                            target_username: vector[1].to_string(),
                        }); 
                    }
                    
                    //player creates room
                    else if vector[0] == "/createroom"{
                        self.addr.do_send(games::CreateRoom{
                            gameid: vector[1].to_string(),
                            username: self.username.clone(),
                            addr: ctx.address().clone(),
                            gamesize: vector[2].to_string().parse::<u8>().unwrap(),
                            mines: vector[3].to_string().parse::<u8>().unwrap(),
                            maxplayers: vector[4].to_string().parse::<u8>().unwrap(),
                        });
                        self.gameid = vector[1].to_string();
                    }

                    //player sends move data
                    else if vector[0] == "/move"{
                        self.addr.do_send(games::MakeMove{
                            gameid: self.gameid.clone(),
                            username: self.username.clone(),
                            addr: ctx.address().clone(),
                            x: vector[1].parse::<usize>().unwrap(),
                            y: vector[2].parse::<usize>().unwrap(),
                        });
                    }
                    else if vector[0] == "/flag"{
                        self.addr.do_send(games::PlaceFlag{
                            gameid: self.gameid.clone(),
                            name: self.username.clone(),
                            addr: ctx.address().clone(),
                            x: vector[1].parse::<usize>().unwrap(),
                            y: vector[2].parse::<usize>().unwrap(),
                        });
                    }
                }
            },
            Ok(ws::Message::Close(_)) => {
                self.addr.do_send(games::DisconnectFromGame{
                    username: self.username.clone(),
                    addr: ctx.address().clone(),
                    gameid: self.gameid.clone(),
                });
                self.addr.do_send(games::Offline{
                    username: self.username.clone(),
                    addr: ctx.address().clone(),
                });
                ctx.stop();
            },
            _ => {
                self.addr.do_send(games::DisconnectFromGame{
                    username: self.username.clone(),
                    addr: ctx.address().clone(),
                    gameid: self.gameid.clone(),
                });
                self.addr.do_send(games::Offline{
                    username: self.username.clone(),
                    addr: ctx.address().clone(),
                });
                ctx.stop();
            },
        }
    }
}

