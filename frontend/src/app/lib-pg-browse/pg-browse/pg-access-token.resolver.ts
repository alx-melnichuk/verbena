import { inject } from "@angular/core";
import { ActivatedRouteSnapshot, ResolveFn, RouterStateSnapshot } from "@angular/router";
import { SessionSrv } from "../../common/session-srv";

export const pgAccessTokenResolver: ResolveFn<string | null>
    = (_route: ActivatedRouteSnapshot, _state: RouterStateSnapshot) => {
        const sessionSrv = inject(SessionSrv);
        return sessionSrv.getAccessToken();
    };
