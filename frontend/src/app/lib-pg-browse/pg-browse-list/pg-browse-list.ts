import { CommonModule } from "@angular/common";
import { ChangeDetectionStrategy, ChangeDetectorRef, Component, inject, OnDestroy, OnInit, ViewEncapsulation } from "@angular/core";
import { HttpErrorResponse } from "@angular/common/http";
import { ActivatedRoute, Params, Router } from "@angular/router";
import { LocaleSrv } from "../../common/locale-srv";
import { G_BROWSE_LIVE, G_BROWSE_PAGE, G_BROWSE_TAG, ROUTE_BROWSE_LIST } from "../../common/routes";
import { SessionSrv } from "../../common/session-srv";
import { ItemViewPage } from "../../components/view-list-by-pages/view-list-by-pages";
import { AlertSrv } from "../../lib-dialog/alert-srv";
import { StreamTagDto, PageStreamAndTagsDto, StreamDtoUtil, PageStreamTagDto } from "../../lib-stream/stream-dto";
import { StreamSrv } from "../../lib-stream/stream-srv";
import { HttpErrorUtil } from "../../utils/http-error.util";
import { PanelBrowseList } from "../panel-browse-list/panel-browse-list";
import { Subscription } from "rxjs";

const POP_TAGS_LIMIT_DEF = 20;
const STRM_LIMIT_DEF = 10;

@Component({
    selector: "app-pg-browse-list",
    exportAs: "appPgBrowseList",
    standalone: true,
    imports: [CommonModule, PanelBrowseList],
    templateUrl: "./pg-browse-list.html",
    styleUrl: "./pg-browse-list.scss",
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class PgBrowseList implements OnInit, OnDestroy {
    private alertSrv: AlertSrv = inject(AlertSrv);
    private activatedRoute: ActivatedRoute = inject(ActivatedRoute);
    private router: Router = inject(Router);
    private changeDetector: ChangeDetectorRef = inject(ChangeDetectorRef);
    public localeSrv: LocaleSrv = inject(LocaleSrv);
    private sessionSrv: SessionSrv = inject(SessionSrv);
    private streamSrv: StreamSrv = inject(StreamSrv);

    public isLive: boolean | null | undefined = null;
    public tag: string | null | undefined = null;
    public page: number = 0;

    public popTagIsLoading: boolean | null | undefined;
    public popTagList: StreamTagDto[] = [];

    public strmCrdIsLoading: boolean | null | undefined;
    public strmCrdIsReset: boolean | null | undefined;
    public strmCrdItemPage: ItemViewPage | null | undefined;

    public userId: number = this.sessionSrv.getUser()?.id || -1;

    private queryParamsSub: Subscription | undefined;
    private pages: number = 0;

    ngOnInit(): void {
        // Get a list of popular tags.
        this.doLoadPopularTags();

        this.queryParamsSub = this.activatedRoute.queryParams.subscribe((params: Params) => {
            const isLive = params[G_BROWSE_LIVE] == "true";
            const tag = params[G_BROWSE_TAG];
            const page = parseInt(params[G_BROWSE_PAGE] || "0", 10);
            this.loadStreamPage(isLive, tag, page, 0);
        });
    }

    ngOnDestroy(): void {
        this.queryParamsSub?.unsubscribe();
    }

    // ** Public API **

    public doLoadPopularTags(): Promise<void> {
        this.popTagIsLoading = true;
        return this.streamSrv.getStreamsPopularTags({ sortColumn: "countlinks", sortDesc: true, limit: POP_TAGS_LIMIT_DEF })
            .then((response: PageStreamTagDto | HttpErrorResponse) => {
                this.popTagList = (response as PageStreamTagDto).list;
            })
            .catch((err: HttpErrorResponse) => {
                const message = HttpErrorUtil.mapErrMsgObjs(err.status, err.error)?.[0].msg || "error.server_api_call";
                this.alertSrv.showError(message, "pg-browse-list?.error_get_popular_tags");
                throw err;
            })
            .finally(() => {
                this.popTagIsLoading = false;
                this.changeDetector.markForCheck();
            });

    }

    public doLoadStrmCrdPage(isLive: boolean | null, tag: string | null, page: number, limit: number): void {
        const queryParams: Params = {};
        if (!!isLive) {
            queryParams[G_BROWSE_LIVE] = true;
        }
        if (!!tag) {
            queryParams[G_BROWSE_TAG] = tag;
        }
        if (!!page && page > 0) {
            queryParams[G_BROWSE_PAGE] = page;
        }
        this.router.navigate([ROUTE_BROWSE_LIST], { queryParams });
    }

    public doActionView(streamId: number): void {
        if (!!streamId) {
            this.streamSrv.redirectToStreamViewPage(streamId);
        }
    }

    // ** Private API **

    public loadStreamPage(isLive: boolean | null, tag: string | null, page1: number, limit1: number): Promise<void> {
        if (page1 > 0 && this.pages > 0 && page1 > this.pages) {
            return Promise.resolve();
        }
        const live = isLive ? true : undefined;
        const page = page1 > 0 ? page1 : 1;
        const limit = limit1 > 0 ? limit1 : STRM_LIMIT_DEF;
        this.strmCrdIsReset = page1 < 1;
        this.strmCrdIsLoading = true;
        return this.streamSrv.getStreamsWithTagByPage(tag, live, page, limit)
            .then((response: PageStreamAndTagsDto | HttpErrorResponse | undefined) => {
                const streams = (response as PageStreamAndTagsDto);
                this.strmCrdItemPage = { list: StreamDtoUtil.createList(streams.list), page: streams.page, limit };
                this.isLive = isLive;
                this.tag = tag;
                this.page = page;
                this.pages = streams.pages;
            })
            .catch((err: HttpErrorResponse) => {
                const message = HttpErrorUtil.mapErrMsgObjs(err.status, err.error)?.[0].msg || "error.server_api_call";
                this.alertSrv.showError(message, "pg-browse-list?.error_get_streams_by_tag");
                throw err;
            })
            .finally(() => {
                this.strmCrdIsLoading = false;
                this.changeDetector.markForCheck();
            });
    }
}
