import { AuthorizationUtil } from '../utils/authorization.util';
import { R_ROOT } from './routes';

export interface MainMenu {
    name: string;
    link: string;
};

export class MainMenuUtil {
    public static getList(isAuth: boolean, list: string[]): MainMenu[] {
        const result: MainMenu[] = [];
        const prefix = '/' + R_ROOT + '/';
        for (let index = 0; index < list.length; index++) {
            const menuLink = list[index];
            if (!menuLink.startsWith(prefix)) { continue; }
            const isAuthRequired = AuthorizationUtil.isAuthorizationRequired(menuLink);
            const isAuthDenied = AuthorizationUtil.isAuthorizationDenied(menuLink);
            const name = menuLink.slice(prefix.length);
            const menuName = name.replaceAll('/', '_');
            if ((isAuth && isAuthRequired) || (!isAuth && isAuthDenied) || (!isAuthRequired && !isAuthDenied)) {
                result.push({ name: menuName, link: menuLink });
            }
        }
        return result;
    }
}