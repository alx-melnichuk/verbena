import { ActivatedRouteSnapshot, ResolveFn, Router, RouterStateSnapshot } from '@angular/router';
import { HttpErrorResponse } from '@angular/common/http';
import { inject } from '@angular/core';

import { ROUTE_LOGIN } from '../common/routes';
import { ProfileService } from '../lib-profile/profile.service';
import { ProfileDto } from '../lib-profile/profile-api.interface';

export const pgProfileResolver: ResolveFn<ProfileDto | HttpErrorResponse | undefined> = 
(_route: ActivatedRouteSnapshot, _state: RouterStateSnapshot) => {
  const router = inject(Router);
  const profileService: ProfileService = inject(ProfileService);
  const profileDto: ProfileDto | null = profileService.profileDto;

  if (profileDto == null) {
    return router.navigateByUrl(ROUTE_LOGIN).then(() => Promise.resolve(undefined));
  } else {
    return Promise.resolve(profileDto);
  }
};
