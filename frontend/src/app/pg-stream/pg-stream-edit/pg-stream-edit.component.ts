import { CommonModule } from '@angular/common';
import { HttpErrorResponse } from '@angular/common/http';
import { ChangeDetectionStrategy, ChangeDetectorRef, Component, inject, OnInit, ViewEncapsulation } from '@angular/core';
import { ActivatedRoute, Router } from '@angular/router';
import { TranslateService } from '@ngx-translate/core';
import { Observable } from 'rxjs';

import { IDeactivatePage } from 'src/app/common/deactivate-page.guard';
import { ROUTE_STREAM_EDIT, ROUTE_STREAM_LIST } from 'src/app/common/routes';
import { DialogService } from 'src/app/lib-dialog/dialog.service';
import { StreamDto, UpdateStreamFileDto } from 'src/app/lib-stream/stream-api.interface';
import { StreamConfigDto } from 'src/app/lib-stream/stream-config.interface';

import { StreamEditComponent } from 'src/app/lib-stream/stream-edit/stream-edit.component';
import { StreamService } from 'src/app/lib-stream/stream.service';
import { HttpErrorUtil } from 'src/app/utils/http-error.util';

@Component({
    selector: 'app-pg-stream-edit',
    standalone: true,
    imports: [CommonModule, StreamEditComponent],
    templateUrl: './pg-stream-edit.component.html',
    styleUrl: './pg-stream-edit.component.scss',
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush
})
export class PgStreamEditComponent implements OnInit, IDeactivatePage {
    private changeDetector: ChangeDetectorRef = inject(ChangeDetectorRef);
    private dialogService: DialogService = inject(DialogService);
    private goBackToRoute: string = ROUTE_STREAM_LIST;
    private isChangeData: boolean = false;
    private route: ActivatedRoute = inject(ActivatedRoute);
    private router: Router = inject(Router);
    private streamService: StreamService = inject(StreamService);
    private translateService: TranslateService = inject(TranslateService);

    public errMsgs: string[] = [];
    public isLoadStream = false;
    public streamDto: StreamDto | null = this.route.snapshot.data['streamDto'] || null;
    public streamConfigDto: StreamConfigDto | null = this.route.snapshot.data['streamConfigDto'] || null;

    ngOnInit(): void {
        const previousNav = this.router.getCurrentNavigation()?.previousNavigation?.finalUrl?.toString() || '';
        if (!!previousNav && !previousNav.startsWith(ROUTE_STREAM_EDIT)) {
            this.goBackToRoute = previousNav;
        }
    }

    // ** IDeactivatePage **

    public canExit = (): Observable<boolean> | Promise<boolean> | boolean => {
        if (this.isChangeData) {
            const message = this.translateService.instant('pg-stream-edit.have_unsaved_you_want_to_leave');
            return this.dialogService.openConfirmation(message, '', { btnNameCancel: 'buttons.no', btnNameAccept: 'buttons.yes' })
                .then((value) => !!value)
                .catch(() => true);
        }
        return true;
    };

    // ** Public API **

    public doChangeData(): void {
        this.isChangeData = true;
    }

    public doUpdateStream(updateStreamFileDto: UpdateStreamFileDto | null): void {
        if (!updateStreamFileDto) {
            return;
        }
        // Checking if there are any non-empty fields.
        const key = 0; const value = 1; // entry=[key, value] -> entry[0]-key, entry[1]-value
        const is_all_empty = Object.entries(updateStreamFileDto)
            .findIndex((entry) => entry[key] != 'id' && entry[value] !== undefined) == -1;
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
            buffPromise.push(this.streamService.modifyStream(updateStreamFileDto.id, updateStreamFileDto));
        } else {
            buffPromise.push(this.streamService.createStream(updateStreamFileDto));
        }
        Promise.all(buffPromise)
            .then(() => {
                Promise.resolve()
                    .then(() => {
                        this.goBack();
                    });
            })
            .catch((error: HttpErrorResponse) => {
                this.errMsgs = HttpErrorUtil.getMsgs(error);
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
