import { HttpErrorResponse } from "@angular/common/http";
import { inject } from "@angular/core";
import { ActivatedRouteSnapshot, ResolveFn, RouterStateSnapshot } from "@angular/router";
import { StreamConfigDto } from "../stream-config-dto";
import { StreamConfigSrv } from "../stream-config-srv";

export const pgStreamConfigResolver: ResolveFn<StreamConfigDto | HttpErrorResponse | undefined>
    = (_route: ActivatedRouteSnapshot, _state: RouterStateSnapshot) => {
        const streamConfigSrv: StreamConfigSrv = inject(StreamConfigSrv);
        return streamConfigSrv.getConfig();
    };
