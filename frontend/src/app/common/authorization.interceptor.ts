import { HttpRequest, HttpHandler, HttpEvent, HttpInterceptor, HttpErrorResponse } from "@angular/common/http";
import { Injectable, inject } from "@angular/core";
import { Router } from "@angular/router";
import { Subject, Observable, catchError, throwError, take, switchMap } from "rxjs";
import { environment } from "../../environments/environment";
import { TokenUpdate, UserTokenResponseDto } from "../lib-user/user-dto";
import { UserSrv } from "../lib-user/user-srv";
import { LIST_PUBLIC_METHODS } from "./public-methods";
import { ROUTE_LOGIN } from "./routes";
import { SessionSrv } from "./session-srv";
import { Uri } from "./uri";


const CN_BEARER = "Bearer ";

@Injectable({
    providedIn: "root",
})
export class AuthorizationInterceptor implements HttpInterceptor {
    private router: Router = inject(Router);
    private sessionSrv: SessionSrv = inject(SessionSrv);
    private userSrv: UserSrv = inject(UserSrv);

    private refreshTokenInProgress = false;
    private refreshTokenSubject: Subject<boolean> = new Subject();
    // List of public methods that do not require authorization.
    private listPublicMethods: { [key: string]: string } = LIST_PUBLIC_METHODS;
    private tokenUpdateSrv: TokenUpdate = this.userSrv;

    constructor() {
        if (environment.logLevel > 0) { console.log(`AuthorizationInterceptor(); // 4 service`); }
    }

    intercept(request: HttpRequest<unknown>, next: HttpHandler): Observable<HttpEvent<unknown>> {
        request = this.addAuthenticationToken(request);

        return next.handle(request).pipe(
            // tap((evt) => console .log("evt=", evt)),
            catchError((error: HttpErrorResponse) => {
                // If an error occurs when updating the token, then redirect to the login page.
                if (this.refreshTokenInProgress && this.isCheckRefreshToken(request.method, request.url)) {
                    // Clear the authorization token value.
                    this.sessionSrv.removeUserTokens();
                    this.sessionSrv.removeUser();
                    // And you need to go to the "login" tab.
                    window.setTimeout(() => this.router.navigateByUrl(ROUTE_LOGIN, { replaceUrl: true }), 0);
                    return throwError(() => error);
                }
                const refreshToken = this.sessionSrv.getRefreshToken();
                // 401 Unauthorized, 403 Forbidden
                if (error?.status == 401 && !!refreshToken) {
                    // the errors will most likely occur because we have an expired token that we need to refresh.
                    if (!this.refreshTokenInProgress) {
                        this.refreshTokenInProgress = true;
                        // Get a new token.
                        this.refreshAccessToken(refreshToken)
                            .then(() => this.refreshTokenSubject.next(true))
                            .catch((error) => this.refreshTokenSubject.error(error))
                            // When the call to refreshToken completes we reset the "refreshTokenInProgress" to false
                            .finally(() => (this.refreshTokenInProgress = false));
                    }
                    return this.refreshTokenSubject.pipe(
                        take(1),
                        switchMap(() => next.handle(this.addAuthenticationToken(request)))
                    );
                } else {
                    return throwError(() => error);
                }
            })
        ); // as Observable<HttpEvent<unknown>>;
    }

    // ** Private **

    private isCheckRefreshToken(method: string, url: string): boolean {
        return method === "POST" && url === Uri.appUri("appApi://token");
    }

    private addAuthenticationToken(request: HttpRequest<any>): HttpRequest<any> {
        const accessToken = this.sessionSrv.getAccessToken();
        // If the call is to an external domain, then the token is not added.
        let isNotIncludes = !request.url.includes(Uri.appUri("appApi://"));
        let publicMethod = this.listPublicMethods[request.url];
        if (!accessToken || isNotIncludes || publicMethod === request.method) {
            return request;
        }
        return request.clone({ setHeaders: { "Authorization": CN_BEARER + accessToken } });
    }

    private refreshAccessToken(refreshToken: string): Promise<void> {
        return this.tokenUpdateSrv.refreshToken(refreshToken)
            .then((response: UserTokenResponseDto | HttpErrorResponse | undefined) => {
                const res = response as UserTokenResponseDto;
                this.sessionSrv.setUserTokens(res.accessToken, res.refreshToken);
                if (environment.logLevel > 0) { console.log(`refreshAccessToken successful`); }
                return Promise.resolve();
            })
            .catch((err: HttpErrorResponse) => {
                if (environment.logLevel > 0) { console.log(`refreshAccessToken err:`, err); }
                this.sessionSrv.removeUserTokens();
                this.sessionSrv.removeUser();
                return Promise.reject(err);
            });
    }
}
