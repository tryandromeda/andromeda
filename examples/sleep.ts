[1, 2, 3, 4].forEach((i) => {
    Andromeda.sleep(i * 1000).then(() => console.log(`${i}s`));
});
