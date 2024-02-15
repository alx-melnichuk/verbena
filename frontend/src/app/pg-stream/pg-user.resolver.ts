import { ActivatedRouteSnapshot, ResolveFn, Router, RouterStateSnapshot } from '@angular/router';
import { HttpErrorResponse } from '@angular/common/http';
import { inject } from '@angular/core';

import { ROUTE_LOGIN } from '../common/routes';
import { UserDto } from '../entities/user/user-dto';
import { UserService } from '../entities/user/user.service';

export const pgUserResolver: ResolveFn<UserDto | HttpErrorResponse | undefined> = 
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
