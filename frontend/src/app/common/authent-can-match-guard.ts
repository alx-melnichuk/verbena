import { inject } from "@angular/core";
import { CanActivateFn, ActivatedRouteSnapshot, RouterStateSnapshot, UrlTree, Router } from "@angular/router";
import { Observable } from "rxjs";
import { RedirectSrv } from "./redirect-srv";
import { ROUTE_LOGIN } from "./routes";
import { SessionSrv } from "./session-srv";
import { environment } from "../../environments/environment";

export const authentCanMatchGuard: CanActivateFn = (
    _route: ActivatedRouteSnapshot, state: RouterStateSnapshot
): Observable<boolean | UrlTree> | Promise<boolean | UrlTree> | boolean | UrlTree => {
    const redirectSrv: RedirectSrv = inject(RedirectSrv);
    const router: Router = inject(Router);
    const sessionSrv: SessionSrv = inject(SessionSrv);

    if (environment.logLevel > 0) {
        console.log(`authentCanMatchGuard() !!sessionSrv.getUser(): ${!!sessionSrv.getUser()}, user.id: ${sessionSrv.getUser()?.id}`);
    }
    if (!!sessionSrv.getUser()) {
        return true;
    }
    // Save the link address to navigate to after login.
    redirectSrv.setUrlAfterLogin(window.location.pathname);
    // Return a UrlTree to redirect without throwing
    return router.createUrlTree([ROUTE_LOGIN]);
};
