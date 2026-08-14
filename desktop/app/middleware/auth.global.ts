export default defineNuxtRouteMiddleware((to) => {
  const authenticated = useState<boolean>("auth:authenticated", () => false);
  const isAuthRoute = to.path.startsWith("/auth");

  if (!authenticated.value) {
    if (!isAuthRoute) {
      return navigateTo("/auth/login");
    }
  } else if (isAuthRoute) {
    return navigateTo("/");
  }
});
