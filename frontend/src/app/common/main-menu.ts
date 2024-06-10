import { AuthorizationUtil } from "../utils/authorization.util";
import { ROUTE_ABOUT, ROUTE_LOGIN, ROUTE_SIGNUP, ROUTE_PROFILE, ROUTE_STREAM_LIST, ROUTE_STREAM_CREATE } from "./routes";

export interface MainMenu {
  name: string;
  link: string;
};

export const mainMenuList = [
  { name: "main_menu.about", link: ROUTE_ABOUT },
  { name: "main_menu.login", link: ROUTE_LOGIN },
  { name: "main_menu.signup", link: ROUTE_SIGNUP },
  { name: "main_menu.profile", link: ROUTE_PROFILE },
  { name: "main_menu.my_streams", link: ROUTE_STREAM_LIST },
  { name: "main_menu.create_stream", link: ROUTE_STREAM_CREATE },
];

export class MainMenuUtil {
  public static getList(currentRoute: string, isAuth: boolean, list?: MainMenu[] | undefined): MainMenu[] {
    const result: MainMenu[] = [];
    const list2 = list || mainMenuList;
    for (let index = 0; index < list2.length; index++) {
        const item: MainMenu = list2[index];
        const isAuthorizationRequired = AuthorizationUtil.isAuthorizationRequired(item.link);
        const isAuthorizationDenied = AuthorizationUtil.isAuthorizationDenied(item.link);
        if ((isAuth && isAuthorizationRequired) || (!isAuth && isAuthorizationDenied)
          || (!isAuthorizationRequired && !isAuthorizationDenied)) {
          result.push(item);
        }
    }
    return result;
  }
}