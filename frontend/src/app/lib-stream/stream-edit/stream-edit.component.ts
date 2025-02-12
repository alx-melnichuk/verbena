import { ChangeDetectionStrategy, ChangeDetectorRef, Component, HostBinding, ViewEncapsulation } from '@angular/core';
import { CommonModule } from '@angular/common';
import { SpinnerComponent } from 'src/app/components/spinner/spinner.component';
import { PanelStreamEditorComponent } from '../panel-stream-editor/panel-stream-editor.component';
import { HttpErrorResponse } from '@angular/common/http';
import { ActivatedRoute, Router } from '@angular/router';

import { LocaleService } from 'src/app/common/locale.service';
import { ROUTE_STREAM_EDIT, ROUTE_STREAM_LIST } from 'src/app/common/routes';
import { HttpErrorUtil } from 'src/app/utils/http-error.util';
import { StreamService } from '../stream.service';
import { StreamDto, UpdateStreamFileDto } from '../stream-api.interface';
import { StreamConfigDto } from '../stream-config.interface';

@Component({
    selector: 'app-stream-edit',
    standalone: true,
    imports: [CommonModule, SpinnerComponent, PanelStreamEditorComponent],
    templateUrl: './stream-edit.component.html',
    styleUrl: './stream-edit.component.scss',
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush
})
export class StreamEditComponent {
    public isLoadDataStream = false;
    public streamDto: StreamDto;
    public streamConfigDto: StreamConfigDto;
    public errMsgs: string[] = [];

    private goBackToRoute: string = ROUTE_STREAM_LIST;

    @HostBinding('class.global-scroll')
    public get classGlobalScrollVal(): boolean {
        return true;
    }

    constructor(
        private changeDetectorRef: ChangeDetectorRef,
        private route: ActivatedRoute,
        private router: Router,
        public localeService: LocaleService,
        private streamService: StreamService,
    ) {
        this.streamDto = this.route.snapshot.data['streamDto'];
        this.streamConfigDto = this.route.snapshot.data['streamConfigDto'];

        const previousNav = this.router.getCurrentNavigation()?.previousNavigation?.finalUrl?.toString() || '';
        if (!!previousNav && !previousNav.startsWith(ROUTE_STREAM_EDIT)) {
            this.goBackToRoute = previousNav;
        }
    }

    // ** Public API **

    public doUpdateStream(updateStreamFileDto: UpdateStreamFileDto): void {
        if (!updateStreamFileDto) {
            return;
        }
        const is_all_empty = Object.entries(updateStreamFileDto).findIndex((entry) => entry[0] != 'id' && entry[1] !== undefined) == -1;
        if (is_all_empty) { // If no fields are specified for update, exit.
            this.goBack();
            return;
        }

        const buffPromise: Promise<unknown>[] = [];
        this.isLoadDataStream = true;

        if (!!updateStreamFileDto.id) {
            buffPromise.push(this.streamService.modifyStream(updateStreamFileDto.id, updateStreamFileDto));
        } else {
            buffPromise.push(this.streamService.createStream(updateStreamFileDto));
        }
        Promise.all(buffPromise)
            .then((responses) => {
                Promise.resolve()
                    .then(() => {
                        this.goBack();
                    });
            })
            .catch((error: HttpErrorResponse) => {
                this.errMsgs = HttpErrorUtil.getMsgs(error);
                throw error;
            })
            .finally(() => {
                this.isLoadDataStream = false;
                this.changeDetectorRef.markForCheck();
            });
    }

    // ** Private API **

    private goBack() {
        window.setTimeout(() => {
            this.router.navigateByUrl(ROUTE_STREAM_LIST);
        }, 0);
    }
}
