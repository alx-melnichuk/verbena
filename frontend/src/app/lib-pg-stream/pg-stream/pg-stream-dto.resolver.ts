import { HttpErrorResponse } from "@angular/common/http";
import { inject } from "@angular/core";
import { ActivatedRouteSnapshot, ResolveFn, Router, RouterStateSnapshot } from "@angular/router";
import { P_STREAM_ID, E_STREAM_CREATE, E_STREAM_EDIT } from "../../common/routes";
import { StreamDto } from "../stream-dto";
import { StreamSrv } from "../stream-srv";


function goToPageNotFound(router: Router): Promise<undefined> {
    return router.navigateByUrl("/technical/not-found").then(() => Promise.resolve(undefined));
}

export const pgStreamDtoResolver: ResolveFn<StreamDto | null | HttpErrorResponse | undefined>
    = (route: ActivatedRouteSnapshot, _state: RouterStateSnapshot) => {
        const router = inject(Router);
        const streamSrv: StreamSrv = inject(StreamSrv);

        const url = route.url[0];
        const streamIdStr = route.paramMap.get(P_STREAM_ID);
        const streamId = parseInt(streamIdStr || "-1", 10);
        if (E_STREAM_CREATE === url.path) {
            const streamIdForDuplicationStr = route.queryParamMap.get("id");
            const streamIdForDuplication = parseInt(streamIdForDuplicationStr || "-1", 10);
            // If the "ID" parameter is specified, then this stream for duplication.
            if (streamIdForDuplication > 0) {
                return streamSrv.getStream(streamIdForDuplication)
                    .then((response: StreamDto | HttpErrorResponse | undefined) => {
                        (response as StreamDto).id = -1;
                        return response;
                    })
                    .catch((err) =>
                        goToPageNotFound(router));
            } else {
                return Promise.resolve(null);
            }
        } else if (E_STREAM_EDIT === url.path && !!streamId) {
            return streamSrv.getStream(streamId)
                .catch((err) =>
                    goToPageNotFound(router));
        } else {
            return goToPageNotFound(router);
        }
    };
