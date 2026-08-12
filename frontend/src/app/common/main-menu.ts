import { AUTHENT_DENIED, AUTHENT_REQUIRED, R_ROOT } from "./routes";

export interface MainMenu {
    name: string;
    link: string;
};

export class MainMenuUtil {
    public static isAuthorizationRequired(currentRoute: string): boolean {
        return AUTHENT_REQUIRED.findIndex((item) => currentRoute.startsWith(item)) > -1;
    }
    public static isAuthorizationDenied(currentRoute: string): boolean {
        return AUTHENT_DENIED.findIndex((item) => currentRoute.startsWith(item)) > -1;
    }
    public static getList(isAuth: boolean, list: string[]): MainMenu[] {
        const result: MainMenu[] = [];
        const prefix = "/" + R_ROOT + "/";
        for (let index = 0; index < list.length; index++) {
            const menuLink = list[index];
            if (!menuLink.startsWith(prefix)) { continue; }
            const isAuthRequired = this.isAuthorizationRequired(menuLink);
            const isAuthDenied = this.isAuthorizationDenied(menuLink);
            const name = menuLink.slice(prefix.length);
            const menuName = name.replaceAll("/", "_");
            if ((isAuth && isAuthRequired) || (!isAuth && isAuthDenied) || (!isAuthRequired && !isAuthDenied)) {
                result.push({ name: menuName, link: menuLink });
            }
        }
        return result;
    }
}
