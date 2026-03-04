import { HttpErrorResponse } from "@angular/common/http";
import { inject } from "@angular/core";
import { ResolveFn, ActivatedRouteSnapshot, RouterStateSnapshot } from "@angular/router";
import { ProfileConfigDto } from "../profile-config-dto";
import { ProfileConfigSrv } from "../profile-config-srv";

export const pgProfileConfigResolver: ResolveFn<ProfileConfigDto | HttpErrorResponse | undefined>
    = (_route: ActivatedRouteSnapshot, _state: RouterStateSnapshot) => {
        const profileConfigService: ProfileConfigSrv = inject(ProfileConfigSrv);
        return profileConfigService.getConfig();
    };
