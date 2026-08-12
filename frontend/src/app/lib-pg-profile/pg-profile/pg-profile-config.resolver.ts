import { HttpErrorResponse } from "@angular/common/http";
import { inject } from "@angular/core";
import { ResolveFn, ActivatedRouteSnapshot, RouterStateSnapshot } from "@angular/router";
import { ProfileConfigDto } from "../profile-config-dto";
import { ProfileConfigSrv } from "../profile-config-srv";

export const pgProfileConfigResolver: ResolveFn<ProfileConfigDto | HttpErrorResponse | undefined>
    = (_route: ActivatedRouteSnapshot, _state: RouterStateSnapshot) => {
        const profileConfigSrv: ProfileConfigSrv = inject(ProfileConfigSrv);
        return profileConfigSrv.getConfig();
    };
