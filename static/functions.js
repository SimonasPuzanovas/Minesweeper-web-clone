//GAMELIST
function createGameListItem(data){
    //room name element
    let roomname = document.createElement('h1');
    roomname.textContent = data[0];

    //player number element
    let player_num = document.createElement('h1');
    player_num.textContent = data[1] + "/" + data[2];
    if (data[1] == data[2]){
        player_num.style.color = "red";
    }
    else {player_num.style.color = "green";}
    player_num.classList.add("player-num"); 

    //gamelist room container element
    let div = document.createElement('div');
    div.classList.add("gameid"); 

    div.appendChild(roomname);
    div.appendChild(player_num);
    gamelist.appendChild(div);

}
async function get_gamelist(){
    const gameidd = document.querySelectorAll(".gameid");
    gameidd.forEach(id => {
        id.remove();
    })

    await fetch('/gamelist.json').then(resp => resp.json()).then(data => {
        gamelist_event(data);
    })

}

function gamelist_event(data){
    console.log(data);
    for(let i = 0; i < data.games.length; i++){
        createGameListItem(data.games[i]);
    }

    //gameid click event
    for (var i = 0; i < gameid.length; i++){
        gameid[i].onclick = function (e) {
            socket.send("/joinroom " + e.target.firstChild.textContent);
        }
    }

    const no_rooms = document.querySelector(".norooms");
    if (data.games.length == 0){
        no_rooms.style.display = "block";
    }
    else{
        no_rooms.style.display = "none";
    }
}

function playerlist_update(data){
    const player = document.querySelectorAll(".playerlist-player"); 
        player.forEach(e => {
            e.remove();
        })

    for (var i = 1; i < data.length; i += 2){
        let playername = document.createElement('h2');
        let kick = document.createElement('h3');
        kick.classList.add("playerlist-player-kick");
        kick.textContent = "KICK";
        kick.setAttribute("name", data[i]);

        //first in the slot is moderator
        if (i == 1) {
                playername.style.color = data[i + 1];
                playername.textContent = data[i] + " (Moderator)";
        }
        else {
            //name
            playername.textContent = data[i];
            //color
            playername.style.color = data[i + 1];
        }

        playername.classList.add("playerlist-player-name");

        let player = document.createElement("div");
        player.classList.add("playerlist-player");
        player.appendChild(playername);

        if(moderator) {
                player.appendChild(kick);
        }

        playerlist_container.appendChild(player);
    }

    const kick_button = document.querySelectorAll(".playerlist-player-kick")

    kick_button.forEach(button => {
        button.onclick = function() {
            socket.send("/kick " + button.getAttribute("name"));
        }
    })

    const name = document.querySelectorAll(".playerlist-player-name")

    name.forEach(namee => {
        namee.onclick = function() {
            //split in case it's moderator which has name + (moderator)
            if (namee.textContent.split(" ")[0] == my_username){
                socket.send("/changecolor");
            }
        }
    })

}

//UPDATE ELEMENT SIZES DEPENDING ON THE CLIENT VIEWPORT
function update_el_sizes(){
    const vw = window.innerWidth;
    const vh = window.innerHeight;
    //update game size
    if (vw < vh){
        tile_container.style.width = vw - sizee + "px";
        tile_container.style.height = vw - sizee + "px";
    }
    else if (vw > vh){
        tile_container.style.width = vh - sizee + "px";
        tile_container.style.height = vh - sizee + "px";
    }
}

//hamburger related

function ham_menu_on(){
    ham_menu.style.transform = "translateX(0)";
    ham_menu.style.opacity = 1;
    ham_switch = true;
}

function ham_menu_off(){
    ham_menu.style.transform = "translateX(-100%)";
    ham_menu.style.opacity = 0;
    ham_switch = false;
}


