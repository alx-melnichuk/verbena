import { inject } from "@angular/core";
import { ActivatedRouteSnapshot, ResolveFn, RouterStateSnapshot } from "@angular/router";
import { SessionSrv, User } from "../../common/session-srv";

export const pgUserResolver: ResolveFn<User | null>
    = (_route: ActivatedRouteSnapshot, _state: RouterStateSnapshot) => {
        const sessionSrv: SessionSrv = inject(SessionSrv);

        return sessionSrv.getUser();
    };
