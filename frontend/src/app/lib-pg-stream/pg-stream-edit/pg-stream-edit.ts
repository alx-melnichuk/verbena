import { CommonModule } from "@angular/common";
import { HttpErrorResponse } from "@angular/common/http";
import { ChangeDetectionStrategy, ChangeDetectorRef, Component, inject, OnDestroy, OnInit, ViewEncapsulation } from "@angular/core";
import { ActivatedRoute, Router } from "@angular/router";
import { LocaleSrv } from "../../common/locale-srv";
import { ROUTE_STREAM_LIST, ROUTE_STREAM_EDIT } from "../../common/routes";
import { HasUnsavedChanges, UnsavedDeActivateUtil } from "../../common/unsaved-de-activate-page-guard";
import { Spinner } from "../../components/spinner/spinner";
import { ErrMsgObj, HttpErrorUtil } from "../../utils/http-error.util";
import { PanelStreamEditor } from "../panel-stream-editor/panel-stream-editor";
import { StreamConfigDto } from "../stream-config-dto";
import { StreamDto, UpdateStreamFileDto } from "../stream-dto";
import { StreamSrv } from "../stream-srv";

@Component({
    selector: "app-pg-stream-edit",
    exportAs: "appPgStreamEdit",
    standalone: true,
    imports: [CommonModule, Spinner, PanelStreamEditor],
    templateUrl: "./pg-stream-edit.html",
    styleUrl: "./pg-stream-edit.scss",
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class PgStreamEdit implements OnInit, OnDestroy, HasUnsavedChanges {
    private changeDetector: ChangeDetectorRef = inject(ChangeDetectorRef);
    public localeSrv: LocaleSrv = inject(LocaleSrv);
    private route: ActivatedRoute = inject(ActivatedRoute);
    private router: Router = inject(Router);
    private streamSrv: StreamSrv = inject(StreamSrv);

    public errMsgObjs: ErrMsgObj[] = [];
    public isChangeData: boolean = false;
    public isLoadStream = false;
    public streamDto: StreamDto | null = this.route.snapshot.data["streamDto"] || null;
    public streamConfigDto: StreamConfigDto | null = this.route.snapshot.data["streamConfigDto"] || null;

    private goBackToRoute: string = ROUTE_STREAM_LIST;

    ngOnInit(): void {
        const previousNav = this.router.getCurrentNavigation()?.previousNavigation?.finalUrl?.toString() || ""; // ?
        if (!!previousNav && !previousNav.startsWith(ROUTE_STREAM_EDIT)) {
            this.goBackToRoute = previousNav;
        }
        // Set a confirmation string when attempting to leave a page with unsaved data.
        UnsavedDeActivateUtil.setConfirmText("pg-stream-edit.have_unsaved_you_want_to_leave");
    }

    ngOnDestroy() {
        // Reset a confirmation string when attempting to leave a page with unsaved data.
        UnsavedDeActivateUtil.setConfirmText("");
    }

    // ** HasUnsavedChanges **
    public hasUnsavedChanges(): boolean {
        return this.isChangeData;
    }
    // ** **

    // ** Public API **

    public doChangeData(isChange: boolean): void {
        this.isChangeData = isChange;
    }

    public doUpdateStream(updateStreamFileDto: UpdateStreamFileDto | null): void {
        if (!updateStreamFileDto) {
            return;
        }
        // Checking if there are any non-empty fields.
        const key = 0; const value = 1; // entry=[key, value] -> entry[0]-key, entry[1]-value
        const is_all_empty = Object.entries(updateStreamFileDto)
            .findIndex((entry) => entry[key] != "id" && entry[value] !== undefined) == -1;
        if (is_all_empty) { // If no fields are specified for update, exit.
            this.goBack();
            return;
        }

        if (this.isChangeData) {
            this.isChangeData = false;
        }
        const buffPromise: Promise<unknown>[] = [];
        this.isLoadStream = true;

        if (!!updateStreamFileDto.id) {
            buffPromise.push(this.streamSrv.modifyStream(updateStreamFileDto.id, updateStreamFileDto));
        } else {
            buffPromise.push(this.streamSrv.createStream(updateStreamFileDto));
        }
        this.errMsgObjs = [];
        Promise.all(buffPromise)
            .then(() => Promise.resolve().then(() => this.goBack()))
            .catch((err: HttpErrorResponse) => {
                this.errMsgObjs = HttpErrorUtil.mapErrMsgObjs(err.status, err.error);
            })
            .finally(() => {
                this.isLoadStream = false;
                this.changeDetector.markForCheck();
            });
    }

    // ** Private API **

    private goBack() {
        window.setTimeout(() => this.router.navigateByUrl(this.goBackToRoute), 0);
    }

}
