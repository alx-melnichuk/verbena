import { HttpErrorResponse } from "@angular/common/http";
import { inject } from "@angular/core";
import { ActivatedRouteSnapshot, ResolveFn, RouterStateSnapshot } from "@angular/router";
import { UserDto } from "../../lib-user/user-dto";
import { UserSrv } from "../../lib-user/user-srv";

export const pgProfileDtoResolver: ResolveFn<UserDto | HttpErrorResponse | undefined>
    = (_route: ActivatedRouteSnapshot, _state: RouterStateSnapshot) => {
        const userSrv: UserSrv = inject(UserSrv);
        return userSrv.getCurrentUser();
    };
