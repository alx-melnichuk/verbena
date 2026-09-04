import { inject } from "@angular/core";
import { CanActivateFn, ActivatedRouteSnapshot, RouterStateSnapshot, UrlTree, Router } from "@angular/router";
import { Observable } from "rxjs";
import { environment as env } from "../../environments/environment";
import { RedirectSrv } from "./redirect-srv";
import { ROUTE_LOGIN } from "./routes";
import { SessionSrv } from "./session-srv";

export const authentCanActivateGuard: CanActivateFn = (
    _route: ActivatedRouteSnapshot, state: RouterStateSnapshot
): Observable<boolean | UrlTree> | Promise<boolean | UrlTree> | boolean | UrlTree => {
    const redirectSrv: RedirectSrv = inject(RedirectSrv);
    const router: Router = inject(Router);
    const sessionSrv: SessionSrv = inject(SessionSrv);

    if (!!sessionSrv.getUser()) {
        return true;
    }
    if (env.logLevel & 4) {
        console.info(`authentCanActivateGuard() !!sessionSrv.getUser(): ${!!sessionSrv.getUser()}, user.id: ${sessionSrv.getUser()?.id}`);
    }
    // Save the link address to navigate to after login.
    redirectSrv.setUrlAfterLogin(window.location.pathname);
    // Return a UrlTree to redirect without throwing
    return router.createUrlTree([ROUTE_LOGIN]);
};
