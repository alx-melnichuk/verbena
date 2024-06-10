import { ActivatedRouteSnapshot, ResolveFn, Router, RouterStateSnapshot } from '@angular/router';
import { HttpErrorResponse } from '@angular/common/http';
import { inject } from '@angular/core';

import { ROUTE_LOGIN } from '../common/routes';
import { UserService } from '../lib-user/user.service';
import { UserDto } from '../lib-user/user-api.interface';

export const pgUserInfoResolver: ResolveFn<UserDto | HttpErrorResponse | undefined> = 
(_route: ActivatedRouteSnapshot, _state: RouterStateSnapshot) => {
  const router = inject(Router);
  const userService: UserService = inject(UserService);
  const userDto: UserDto | null = userService.userInfo;

  if (userDto == null) {
    return router.navigateByUrl(ROUTE_LOGIN).then(() => Promise.resolve(undefined));
  } else {
    return Promise.resolve(userDto);
  }
};
