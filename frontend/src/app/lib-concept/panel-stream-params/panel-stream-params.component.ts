import { ChangeDetectionStrategy, Component, Input, ViewEncapsulation } from '@angular/core';
import { CommonModule } from '@angular/common';
import { TranslatePipe } from '@ngx-translate/core';

import { DateTimeFormatPipe } from 'src/app/common/date-time-format.pipe';
import { DateTimeTimerComponent } from 'src/app/components/date-time-timer/date-time-timer.component';


@Component({
    selector: 'app-panel-stream-params',
    exportAs: 'appPanelStreamParams',
    standalone: true,
    imports: [CommonModule, DateTimeFormatPipe, TranslatePipe, DateTimeTimerComponent],
    templateUrl: './panel-stream-params.component.html',
    styleUrl: './panel-stream-params.component.scss',
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush
})
export class PanelStreamParamsComponent {
    @Input()
    public countOfViewer: number | null | undefined;
    @Input()
    public isShowTimer: boolean | null | undefined;
    @Input()
    public locale: string | null = null;
    @Input() // Time when stream should start.
    public start: Date | null | undefined;
    @Input() // Time when stream was started
    public started: Date | null | undefined;
    @Input() // Time when stream was stopped
    public stopped: Date | null | undefined;

    readonly formatDateTime: Intl.DateTimeFormatOptions = { dateStyle: 'long', timeStyle: 'short' };

}
