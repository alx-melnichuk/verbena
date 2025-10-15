import { CommonModule } from '@angular/common';
import { ChangeDetectionStrategy, Component, inject, Input, OnChanges, SimpleChanges, ViewEncapsulation } from '@angular/core';
import { TranslatePipe, TranslateService } from '@ngx-translate/core';

import { DateTimeFormatPipe } from 'src/app/common/date-time-format.pipe';
import { TimeTrackingComponent } from 'src/app/components/time-tracking/time-tracking.component';

@Component({
    selector: 'app-panel-stream-params',
    exportAs: 'appPanelStreamParams',
    standalone: true,
    imports: [CommonModule, DateTimeFormatPipe, TranslatePipe, TimeTrackingComponent],
    templateUrl: './panel-stream-params.component.html',
    styleUrl: './panel-stream-params.component.scss',
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class PanelStreamParamsComponent implements OnChanges {
    @Input()
    public countOfViewer: number | null | undefined;
    @Input()
    public locale: string | null = null;
    @Input()
    public streamStarttime: Date | null | undefined;  // The stream start time.
    @Input()
    public streamStarted: Date | null | undefined; // The time the stream began.
    @Input()
    public streamPaused: Date | null | undefined; // The time the stream began pausing.
    @Input()
    public streamStopped: Date | null | undefined; // The time the stream stopped.
    @Input()
    public timerActive: boolean | null | undefined;
    @Input()
    public timerIsShow: boolean | null | undefined;
    @Input()
    public timerValue: number | null | undefined;

    readonly formatDateTime: Intl.DateTimeFormatOptions = { dateStyle: 'long', timeStyle: 'short' };

    private translate: TranslateService = inject(TranslateService);

    public labelDays: Record<number, string> = this.createLabelDays(this.translate);

    constructor() {
    }

    ngOnChanges(changes: SimpleChanges): void {
        if (!!changes['locale']) {
            this.labelDays = this.createLabelDays(this.translate);
        }
    }

    // ** Private API **

    private createLabelDays(translate: TranslateService): Record<number, string> {
        const result: Record<number, string> = {};
        for (let idx = -1; idx <= 20; idx++) {
            const key = `panel-stream-params.time-tracking.${idx}`;
            if (translate.instant(key) != key) {
                result[idx] = key;
            }
        }
        return result;
    }
}
