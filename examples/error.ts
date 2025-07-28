function fetchUserData() {
  // Intentional typo to trigger runtime error with source context
  const response = fertch("https://invalid-url-that-does-not-exist.com/api/users");
  const data = response.json();
  return data;
}

fetchUserData();
