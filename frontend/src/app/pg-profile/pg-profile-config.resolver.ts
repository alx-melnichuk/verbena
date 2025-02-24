
import { ActivatedRouteSnapshot, ResolveFn, RouterStateSnapshot } from '@angular/router';
import { HttpErrorResponse } from '@angular/common/http';
import { inject } from '@angular/core';

import { ProfileConfigDto } from '../lib-profile/profile-config.interface';
import { ProfileConfigService } from '../lib-profile/profile-config.service';

export const pgProfileConfigResolver: ResolveFn<ProfileConfigDto | HttpErrorResponse | undefined> =
    (_route: ActivatedRouteSnapshot, _state: RouterStateSnapshot) => {
        const profileConfigService: ProfileConfigService = inject(ProfileConfigService);
        return profileConfigService.getConfig();
    };
