/// <reference path="../runtime/global.d.ts" />

const name = prompt("what is your name?");
Andromeda.sleep(1000)
    .then(() => {
        console.log(`Hello, ${name}`);
    });

