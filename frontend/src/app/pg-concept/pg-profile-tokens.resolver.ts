import { ActivatedRouteSnapshot, ResolveFn, Router, RouterStateSnapshot } from '@angular/router';
import { HttpErrorResponse } from '@angular/common/http';
import { inject } from '@angular/core';

import { ROUTE_LOGIN } from '../common/routes';
import { ProfileService } from '../lib-profile/profile.service';
import { ProfileTokensDto } from '../lib-profile/profile-api.interface';

export const pgProfileTokensResolver: ResolveFn<ProfileTokensDto | HttpErrorResponse | undefined> =
    (_route: ActivatedRouteSnapshot, _state: RouterStateSnapshot) => {
        const router = inject(Router);
        const profileService: ProfileService = inject(ProfileService);
        const profileTokensDto: ProfileTokensDto | null = profileService.profileTokensDto;

        if (profileTokensDto == null) {
            return router.navigateByUrl(ROUTE_LOGIN).then(() => Promise.resolve(undefined));
        } else {
            return Promise.resolve(profileTokensDto);
        }
    };
