import { Injectable } from '@angular/core';
import { HttpRequest, HttpHandler, HttpEvent, HttpInterceptor, HttpErrorResponse } from '@angular/common/http';
import { Observable, throwError, Subject } from 'rxjs';
import { catchError, switchMap, take } from 'rxjs/operators';

import { Uri } from './uri';
import { LIST_PUBLIC_METHODS } from './public-methods';

import { UserService } from '../entities/user/user.service';

const CN_BEARER = 'Bearer ';

// @Injectable()
@Injectable({
  providedIn: 'root',
})
export class AuthorizationInterceptor implements HttpInterceptor {
  private refreshTokenInProgress = false;
  private refreshTokenSubject: Subject<boolean> = new Subject();
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
        if (this.refreshTokenInProgress && this.userService.isCeckRefreshToken(request.method, request.url)) {
            return throwError(() => error);
        }
        if ([401, 403].includes(error?.status) && this.userService.isExistRefreshToken()) {
          // 401 errors are most likely going to be because we have an expired token that we need to refresh.
          if (!this.refreshTokenInProgress) {
            this.refreshTokenInProgress = true;
            // Get a new token.
            this.refreshAccessToken()
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
    return request.clone({ setHeaders: { authorization: CN_BEARER + accessToken } });
  }

  private refreshAccessToken(): Promise<void> {
    return this.userService
      .refreshToken()
      .then(() => Promise.resolve())
      .catch((error) => Promise.reject(error))
  }
}
