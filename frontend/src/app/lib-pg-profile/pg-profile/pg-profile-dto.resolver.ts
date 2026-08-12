import { HttpErrorResponse } from "@angular/common/http";
import { inject } from "@angular/core";
import { ActivatedRouteSnapshot, ResolveFn, RouterStateSnapshot } from "@angular/router";
import { ProfileDto } from "../profile-dto";
import { ProfileSrv } from "../profile-srv";

export const pgProfileDtoResolver: ResolveFn<ProfileDto | HttpErrorResponse | undefined>
    = (_route: ActivatedRouteSnapshot, _state: RouterStateSnapshot) => {
        const profileSrv: ProfileSrv = inject(ProfileSrv);
        return profileSrv.getCurrentProfile();
    };
