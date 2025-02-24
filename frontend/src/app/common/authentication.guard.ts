import { inject } from '@angular/core';
import { ActivatedRouteSnapshot, CanActivateFn, Router, RouterStateSnapshot, UrlTree } from '@angular/router';
import { Observable } from 'rxjs';

import { ROUTE_LOGIN } from './routes';

import { ProfileService } from '../lib-profile/profile.service';

export const authenticationGuard: CanActivateFn = (
    route: ActivatedRouteSnapshot, state: RouterStateSnapshot
): Observable<boolean | UrlTree> | Promise<boolean | UrlTree> | boolean | UrlTree => {
    let router: Router = inject(Router);
    let profileService: ProfileService = inject(ProfileService);

    const urlTreeLogin = router.parseUrl(ROUTE_LOGIN);
    const profileDto = profileService.profileDto;
    return Promise.resolve(!!profileDto ? true : urlTreeLogin);
};
