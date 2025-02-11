import { AUTHORIZATION_DENIED, AUTHORIZATION_REQUIRED } from '../common/routes';

export class AuthorizationUtil {
    /** Checking whether authorization is required for a route.
     * @param currentRoute - current route (window.location.pathname);
     * @returns boolean
     */
    public static isAuthorizationRequired(currentRoute: string): boolean {
        return AUTHORIZATION_REQUIRED.findIndex((item) => currentRoute.startsWith(item)) > -1;
    }

    /** Checking whether authorization is denied for a route.
     * @param currentRoute - current route (window.location.pathname);
     * @returns boolean
     */
    public static isAuthorizationDenied(currentRoute: string): boolean {
        return AUTHORIZATION_DENIED.findIndex((item) => currentRoute.startsWith(item)) > -1;
    }
}
