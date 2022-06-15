const tile_container = document.querySelector(".tile-container");

//gameplay states
var playing = false;
var gamesize = null;

function generate_gamefield(gamesize){

    for (var y = 0; y < gamesize; y++){
        for (var x = 0; x < gamesize; x++){

            var overlay = document.createElement("div");
            overlay.classList.add("cell-overlay");
            overlay.setAttribute("x", x);
            overlay.setAttribute("y", y);
            overlay.setAttribute("overlay", "true");

            var background = document.createElement("div");
            background.classList.add("cell-background");
            background.setAttribute("x", x);
            background.setAttribute("y", y);
            background.setAttribute("background", "true");

            var el = document.createElement("div");
            el.classList.add("cell");
            el.classList.add("tile-unknown");
            el.setAttribute("x", x);
            el.setAttribute("y", y);
            el.style.width = (100 / gamesize) + "%"; 
            el.style.height = (100 / gamesize) + "%"; 

            el.appendChild(overlay);
            el.appendChild(background);

            tile_container.appendChild(el); 
        }
    }
    //after creating elements give them onclick events
    const cells = document.querySelectorAll(".cell");
    cells.forEach(cell => {
        cell.onclick = function(){
            const x = cell.getAttribute("x");
            const y = cell.getAttribute("y");
            if (playing && cell.classList.contains("tile-unknown")){
                socket.send("/move " + x + " " + y);
            }
        }
        cell.oncontextmenu = function(e){
            e.preventDefault();
            const x = cell.getAttribute("x");
            const y = cell.getAttribute("y");
            if (cell.classList.contains("tile-unknown") || cell.classList.contains("tile-flag")){
                socket.send("/flag " + x + " " + y);
            }
        }
    });
    gamesize = gamesize;
}

function update_flags(data){
    const cells = document.querySelectorAll(".cell");
    cells.forEach(cell => {
        if (cell.classList.contains("tile-flag")){
            cell.classList.remove(cell.classList.item(2));
            cell.classList.add("tile-unknown");
        }
    });
     

    for (var i = 1; i < data.length; i += 2){
        const x = data[i];
        const y = data[i + 1];
        var celll = document.querySelector("[x='"+ x + "'][y='" + y + "']");

        celll.classList.remove(celll.classList.item(2));
        celll.classList.add("tile-flag");
    }
}

function has(array, [x, y]){
    for (var j = 0; j < array.length; j++){
        if (x == array[j][0] && y == array[j][1]){
            return true;
        }
    }
    return false;
}

function cell_color_fill(x, y, color){
    var array = new Array();
    var stack = new Array([x, y]);
    var idx = 0;

    var move = setInterval(function(){
        if (stack.length == 0) {clearInterval(move);}
        
        var stack_tmp = stack;
        stack = [];

        for (var i = 0; i < stack_tmp.length; i++){

            var cell_background = document.querySelector(
                    '[x="' + stack_tmp[i][0] + '"][y="' + stack_tmp[i][1] + '"]' + '[background="true"]');

            cell_background.style.backgroundColor = color;

            var cell = document.querySelector(
                    '[x="' + stack_tmp[i][0] + '"][y="' + stack_tmp[i][1] + '"]');

            if (cell.classList.item(2) == "tile-flag"){
                cell_background.style.opacity = 1;
            }

            else if (cell.classList.item(2) == "tile-mine"){
                cell_background.style.opacity = 1;
            }

            else if (cell.classList.item(2) == "tile-0" && !has(array, [stack_tmp[i][0], stack_tmp[i][1]])){
                cell_background.style.opacity = 0.2;

                //right
                if (stack_tmp[i][0] + 1 < gamesize && !has(array, [stack_tmp[i][0], stack_tmp[i][1]])){
                    stack.push([stack_tmp[i][0] + 1, stack_tmp[i][1]]);
                }
                //left
                if (stack_tmp[i][0] - 1 >= 0 && !has(array, [stack_tmp[i][0], stack_tmp[i][1]])){
                    stack.push([stack_tmp[i][0] - 1, stack_tmp[i][1]]);
                }
                //top
                if (stack_tmp[i][1] + 1 < gamesize && !has(array, [stack_tmp[i][0], stack_tmp[i][1] + 1])){
                    stack.push([stack_tmp[i][0], stack_tmp[i][1] + 1]);
                }
                //bottom
                if (stack_tmp[i][1] - 1 >= 0 && !has(array, [stack_tmp[i][0], stack_tmp[i][1] - 1])){
                    stack.push([stack_tmp[i][0], stack_tmp[i][1] - 1]);
                }
                //right bottom
                if (stack_tmp[i][0] + 1 < gamesize && stack_tmp[i][1] + 1 < gamesize && !has(array, [stack_tmp[i][0], stack_tmp[i][1]])){
                    stack.push([stack_tmp[i][0] + 1, stack_tmp[i][1] + 1]);
                }
                //right top
                if (stack_tmp[i][0] + 1 < gamesize && stack_tmp[i][1] - 1 >= 0 && !has(array, [stack_tmp[i][0], stack_tmp[i][1]])){
                    stack.push([stack_tmp[i][0] + 1, stack_tmp[i][1] - 1]);
                }
                //left bottom
                if (stack_tmp[i][0] - 1 >= 0 && stack_tmp[i][1] + 1 < gamesize && !has(array, [stack_tmp[i][0], stack_tmp[i][1]])){
                    stack.push([stack_tmp[i][0] - 1, stack_tmp[i][1] + 1]);
                }
                //left top
                if (stack_tmp[i][0] - 1 >= 0 && stack_tmp[i][1] - 1 >= 0 && !has(array, [stack_tmp[i][0], stack_tmp[i][1]])){
                    stack.push([stack_tmp[i][0] - 1, stack_tmp[i][1] - 1]);
                }

            }
            else {
                cell_background.style.opacity = 0.2;
            }
            array.push([stack_tmp[i][0], stack_tmp[i][1]]);
            idx += 1;
        }
    }, 0) // activate every second
}

function reset_gamefield(){
    const cells = document.querySelectorAll(".cell");
    cells.forEach(cell => {
        cell.classList.remove(cell.classList.item(2));
        cell.classList.add("tile-unknown");
    });

    const cells_background = document.querySelectorAll(".cell-background")
    cells_background.forEach(cell => {
        cell.style.background = "#c0c0c0";
        cell.style.opacity = 0.3;
    });
}

function update_gamefield(data){
    const cells = document.querySelectorAll(".cell");
    var idx = 0;
    cells.forEach(cell => {
        if (data[idx + 1] == "0"){
            cell.classList.remove(cell.classList.item(2));
            cell.classList.add("tile-0");
        }
        else if (data[idx + 1] == "1"){
            cell.classList.remove(cell.classList.item(2));
            cell.classList.add("tile-1");
        }
        else if (data[idx + 1] == "2"){
            cell.classList.remove(cell.classList.item(2));
            cell.classList.add("tile-2");
        }
        else if (data[idx + 1] == "3"){
            cell.classList.remove(cell.classList.item(2));
            cell.classList.add("tile-3");
        }
        else if (data[idx + 1] == "4"){
            cell.classList.remove(cell.classList.item(2));
            cell.classList.add("tile-4");
        }
        else if (data[idx + 1] == "5"){
            cell.classList.remove(cell.classList.item(2));
            cell.classList.add("tile-5");
        }
        else if (data[idx + 1] == "6"){
            cell.classList.remove(cell.classList.item(2));
            cell.classList.add("tile-6");
        }
        else if (data[idx + 1] == "7"){
            cell.classList.remove(cell.classList.item(2));
            cell.classList.add("tile-7");
        }
        else if (data[idx + 1] == "8"){
            cell.classList.remove(cell.classList.item(2));
            cell.classList.add("tile-8");
        }
        else if (data[idx + 1] == "10"){
            cell.classList.remove(cell.classList.item(2));
            cell.classList.add("tile-mine");
        }
        else if (data[idx + 1] == "11"){
            cell.classList.remove(cell.classList.item(2));
            cell.classList.add("tile-unsolvedmine");
        }
        else if (data[idx + 1] == "12"){
            cell.classList.remove(cell.classList.item(2));
            cell.classList.add("tile-solvedmine");
        }
        idx += 1;
    }) 
}

