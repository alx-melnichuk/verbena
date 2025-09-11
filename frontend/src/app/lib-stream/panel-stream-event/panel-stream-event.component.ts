import {
    ChangeDetectionStrategy, Component, EventEmitter, HostBinding, HostListener, Input, Output, ViewEncapsulation
} from '@angular/core';
import { CommonModule } from '@angular/common';
import { TranslatePipe } from '@ngx-translate/core';

import { DateTimeFormatPipe } from 'src/app/common/date-time-format.pipe';
import { LogotypeComponent } from 'src/app/components/logotype/logotype.component';
import { ScrollElemUtil } from 'src/app/utils/scroll-elem.util';

import { StreamDtoUtil, StreamEventDto } from '../stream-api.interface';

@Component({
    selector: 'app-panel-stream-event',
    standalone: true,
    imports: [CommonModule, TranslatePipe, LogotypeComponent, DateTimeFormatPipe],
    templateUrl: './panel-stream-event.component.html',
    styleUrl: './panel-stream-event.component.scss',
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class PanelStreamEventComponent {
    @Input()
    public canEdit = false;
    @Input() // Used to refresh the view.
    public isRefresh: boolean | null | undefined;
    @Input()
    public locale: string | null = null;
    @Input()
    public streamShortDtoList: StreamEventDto[] = [];

    @Output()
    readonly requestNextPage: EventEmitter<void> = new EventEmitter();
    @Output()
    readonly viewStream: EventEmitter<number> = new EventEmitter();
    @Output()
    readonly editStream: EventEmitter<number> = new EventEmitter();

    readonly formatDateTime: Intl.DateTimeFormatOptions = { dateStyle: 'short', timeStyle: 'short' };

    @HostBinding('class.global-scroll')
    public get isGlobalScroll(): boolean { return true; }

    @HostListener('scrollend', ['$event'])
    public doScrollPanel(event: Event): void {
        event.preventDefault();
        event.stopPropagation();
        const elem: Element | null = event.target as Element;
        if (ScrollElemUtil.relativeOffset(elem?.scrollTop, elem?.clientHeight, elem?.scrollHeight) > 0.98) {
            this.requestNextPage.emit();
        }
    }

    constructor() {
    }

    // ** Public API **

    public isFuture(startTime: string | null): boolean | null {
        return StreamDtoUtil.isFuture(startTime);
    }

    public doEditStream(streamId: number): void {
        if (this.canEdit && !!streamId) {
            this.editStream.emit(streamId);
        }
    }

    public doViewStream(streamId: number): void {
        if (!!streamId) {
            this.viewStream.emit(streamId);
        }
    }
}
