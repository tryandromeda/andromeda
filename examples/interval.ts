/// <reference path="../runtime/global.d.ts" />

let i = 0;

let id = setInterval(() => {
    console.log(i)
    i += 1;
    if(i == 5){
        clearInterval(id);
    }
}, 250)
