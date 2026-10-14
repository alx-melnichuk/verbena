import { AUTHENT_DENIED, AUTHENT_REQUIRED, R_ROOT } from "./routes";

export interface ItemData {
    id: number;
    ownId: number | null;
    label: string;
    route?: string | undefined;
    subItems?: ItemData[];
}

export class MainMenuUtil {
    public static isAuthorizationRequired(currentRoute: string): boolean {
        return AUTHENT_REQUIRED.findIndex((item) => currentRoute.startsWith(item)) > -1;
    }
    public static isAuthorizationDenied(currentRoute: string): boolean {
        return AUTHENT_DENIED.findIndex((item) => currentRoute.startsWith(item)) > -1;
    }
    public static getPrefix(): string {
        return "/" + R_ROOT + "/";
    }
    public static getListByAuth(list: string[], isAuth: boolean): string[] {
        const result: string[] = [];
        const prefix = MainMenuUtil.getPrefix();
        for (let index = 0; index < list.length; index++) {
            const route = list[index];
            if (!route.startsWith(prefix)) { continue; }
            const isAuthRequired = this.isAuthorizationRequired(route);
            const isAuthDenied = this.isAuthorizationDenied(route);
            if ((isAuth && isAuthRequired) || (!isAuth && isAuthDenied) || (!isAuthRequired && !isAuthDenied)) {
                result.push(route);
            }
        }
        return result;
    }
    public static getItemDataList(list: string[], separator: string): ItemData[] {
        const result: ItemData[] = [];
        let id = 0;
        for (let index = 0; index < list.length; index++) {
            const routeValue = list[index];
            const buff = routeValue.split(separator);
            let route = "";
            let ownId: number | null = null;
            for (let idx = 0; idx < buff.length; idx++) {
                const label = buff[idx];
                const item = result.find((val) => val.ownId == ownId && val.label == label);
                route = route + (idx == 0 ? "" : separator) + label;
                const isRoute = (idx == 0 || idx == buff.length - 1);
                if (!!item) {
                    ownId = item.id;
                } else {
                    result.push({ id: ++id, ownId, label, route: (isRoute ? route : "") });
                    ownId = id;
                }
            }
        }
        return result;
    }
}
