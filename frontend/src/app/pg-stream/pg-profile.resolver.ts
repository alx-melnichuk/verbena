import { ActivatedRouteSnapshot, ResolveFn, Router, RouterStateSnapshot } from '@angular/router';
import { ProfileDto } from '../lib-profile/profile-api.interface';
import { HttpErrorResponse } from '@angular/common/http';
import { inject } from '@angular/core';
import { ProfileService } from '../lib-profile/profile.service';
import { ROUTE_LOGIN } from '../common/routes';

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
