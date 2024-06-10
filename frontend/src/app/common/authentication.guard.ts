import { inject } from '@angular/core';
import { ActivatedRouteSnapshot, CanActivateFn, Router, RouterStateSnapshot, UrlTree } from '@angular/router';
import { Observable } from 'rxjs';

import { ROUTE_LOGIN } from './routes';

import { UserService } from '../lib-user/user.service';

export const authenticationGuard: CanActivateFn = (
    route: ActivatedRouteSnapshot,
    state: RouterStateSnapshot
): Observable<boolean | UrlTree> | Promise<boolean | UrlTree> | boolean | UrlTree => {
  let router: Router = inject(Router);
  let userService: UserService = inject(UserService);

  const urlTreeLogin = router.parseUrl(ROUTE_LOGIN);
  const userInfo = userService.userInfo;
  return Promise.resolve(!!userInfo ? true : urlTreeLogin);
};
