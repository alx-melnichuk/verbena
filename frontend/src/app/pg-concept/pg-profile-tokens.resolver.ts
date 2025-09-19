import { ActivatedRouteSnapshot, ResolveFn, RouterStateSnapshot } from '@angular/router';
import { HttpErrorResponse } from '@angular/common/http';
import { inject } from '@angular/core';

import { ProfileService } from '../lib-profile/profile.service';
import { UserTokenResponseDto } from '../lib-profile/profile-api.interface';

export const pgProfileTokensResolver: ResolveFn<UserTokenResponseDto | HttpErrorResponse | undefined> =
    (_route: ActivatedRouteSnapshot, _state: RouterStateSnapshot) => {
        const profileService: ProfileService = inject(ProfileService);
        const profileTokensDto: UserTokenResponseDto | undefined = profileService.profileTokensDto || undefined;

        return Promise.resolve(profileTokensDto);
    };
