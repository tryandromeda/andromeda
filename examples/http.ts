let counter = 0;
function serve(path: string) {
  console.log(path)
  return {
    status: 200,
    body: `request from "${path}". request count: ` + counter++
  };
}
