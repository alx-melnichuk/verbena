import { Injectable } from '@angular/core';
import { HttpRequest, HttpHandler, HttpEvent, HttpInterceptor, HttpErrorResponse, HttpHeaders } from '@angular/common/http';
import { Observable, throwError, BehaviorSubject } from 'rxjs';
import { catchError, filter, switchMap, take, tap } from 'rxjs/operators';
import { Uri } from './uri';
import { LIST_PUBLIC_METHODS } from './public-methods';
import { UserService } from '../entities/user/user.service';

// @Injectable()
@Injectable({
  providedIn: 'root',
})
export class AuthorizationInterceptor implements HttpInterceptor {
  private refreshTokenInProgress = false;
  private refreshTokenSubject: BehaviorSubject<boolean | null> = new BehaviorSubject<boolean | null>(null);
  // List of public methods that do not require authorization.
  private listPublicMethods: { [key: string]: string } = LIST_PUBLIC_METHODS;

  constructor(private userService: UserService) {
    console.log(`#3-AuthorizationInterceptor();`); // #-
  }

  intercept(request: HttpRequest<unknown>, next: HttpHandler): Observable<HttpEvent<unknown>> {
    request = this.addAuthenticationToken(request);
    return next.handle(request).pipe(
      // tap((evt) => console .log('evt=', evt)),
      catchError((error: HttpErrorResponse) => {
        // 401 Unauthorized, 403 Forbidden
        if (this.refreshTokenInProgress && request.method === 'POST' && request.url === Uri.appUri('appApi://profile/token')) {
          return throwError(() => error);
        }
        if (error && [401, 403].includes(error.status) && this.userService.isExistRefreshToken()) {
          // 401 errors are most likely going to be because we have an expired token that we need to refresh.
          if (!this.refreshTokenInProgress) {
            this.refreshTokenInProgress = true;
            // Set the refreshTokenSubject to null so that subsequent API calls will wait until the new token has been retrieved
            this.refreshTokenSubject.next(null);
            // Get a new token.
            this.refreshAccessToken()
              .then((success) => this.refreshTokenSubject.next(success))
              // When the call to refreshToken completes we reset the refreshTokenInProgress to false
              // for the next time the token needs to be refreshed
              .finally(() => (this.refreshTokenInProgress = false));
          }
          // If refreshTokenInProgress is true, we will wait until refreshTokenSubject has a non-null value
          // which means the new token is ready and we can retry the request again
          return this.refreshTokenSubject.pipe(
            filter((result) => result !== null),
            take(1),
            switchMap(() => next.handle(this.addAuthenticationToken(request))),
            catchError((error2) => {
              return throwError(() => error2);
            })
          );
        } else {
          return throwError(() => error);
        }
      })
    );
  }

  // ** Private **

  private addAuthenticationToken(request: HttpRequest<any>): HttpRequest<any> {
    const accessToken = this.userService.getAccessToken();
    // If the call is to an external domain, then the token is not added.
    let isNotIncludes = !request.url.includes(Uri.appUri('appApi://'));
    let publicMethod = this.listPublicMethods[request.url];
    if (!accessToken || isNotIncludes || publicMethod === request.method) {
      return request;
    }
    return request.clone({ setHeaders: { authorization: 'Bearer ' + accessToken } });
  }

  private refreshAccessToken(): Promise<boolean> {
    return this.userService
      .refreshToken()
      .then(() => true)
      .catch(() => false);
  }
}
