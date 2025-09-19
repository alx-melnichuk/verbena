import { ActivatedRouteSnapshot, ResolveFn, RouterStateSnapshot } from '@angular/router';
import { HttpErrorResponse } from '@angular/common/http';
import { inject } from '@angular/core';

import { ProfileService } from '../lib-profile/profile.service';
import { ProfileDto } from '../lib-profile/profile-api.interface';

export const pgProfileResolver: ResolveFn<ProfileDto | HttpErrorResponse | undefined> =
    (_route: ActivatedRouteSnapshot, _state: RouterStateSnapshot) => {
        const profileService: ProfileService = inject(ProfileService);
        const profileDto: ProfileDto | undefined = profileService.profileDto || undefined;

        return Promise.resolve(profileDto);
    };
