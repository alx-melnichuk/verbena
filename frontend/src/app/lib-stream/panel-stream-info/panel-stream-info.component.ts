import {
 ChangeDetectionStrategy, Component, ElementRef, EventEmitter, HostBinding, HostListener, Input, Output, ViewEncapsulation
} from '@angular/core';
import { CommonModule } from '@angular/common';
import { MatButtonModule } from '@angular/material/button';
import { MatCardModule } from '@angular/material/card';
import { TranslateModule } from '@ngx-translate/core';

import { DateTimeFormatConfigOptions, DateTimeFormatPipe } from 'src/app/common/date-time-format.pipe';
import { LogotypeComponent  } from 'src/app/components/logotype/logotype.component';
import { ScrollHasMaxUtil   } from 'src/app/utils/scroll-has-max.util';

import { StreamDto } from '../stream-api.interface';

const CN_ScrollPanelTimeout = 200; // milliseconds

@Component({
  selector: 'app-panel-stream-info',
  standalone: true,
  imports: [CommonModule, MatButtonModule, MatCardModule, TranslateModule, LogotypeComponent, DateTimeFormatPipe],
  templateUrl: './panel-stream-info.component.html',
  styleUrl: './panel-stream-info.component.scss',
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush
})
export class PanelStreamInfoComponent {
  @Input()
  public canDuplicate = false;
  @Input()
  public canEdit = false;
  @Input()
  public canDelete = false;
  @Input()
  public isFuture = false;
  @Input()
  public locale: string | null = null;
  @Input()
  public streamList: StreamDto[] = [];
  @Input()
  public set isScrollToTop(value: boolean) {
    if (value) {
      this.scrollToTop();
    }
  }
  public get isScrollToTop(): boolean {
    return false;
  }

  @Output()
  readonly requestNextPage: EventEmitter<void> = new EventEmitter();
  @Output()
  readonly redirectToStream: EventEmitter<number> = new EventEmitter();
  @Output()
  readonly actionDuplicate: EventEmitter<number> = new EventEmitter();
  @Output()
  readonly actionEdit: EventEmitter<number> = new EventEmitter();
  @Output()
  readonly actionDelete: EventEmitter<{ id: number, title: string }> = new EventEmitter();

  readonly formatDate: DateTimeFormatConfigOptions = { dateStyle: 'long' };
  readonly formatDateTime: DateTimeFormatConfigOptions = { dateStyle: 'long', timeStyle: 'short' };

  @HostBinding('class.global-scroll')
  public get isGlobalScroll(): boolean { return true; }

  private timerScrollPanel: any = null;

  @HostListener('scroll', ['$event'])
  public doScrollPanel(event:Event):void {
    event.preventDefault();
    event.stopPropagation();
    const elem: Element | null = event.target as Element;
    if (elem != null) {
      if (this.timerScrollPanel !== null) {
        clearTimeout(this.timerScrollPanel);
      }
      this.timerScrollPanel = setTimeout(() => {
        this.timerScrollPanel = null;
        if (ScrollHasMaxUtil.check(elem?.scrollTop, elem?.clientHeight, elem?.scrollHeight)) {
          this.requestNextPage.emit();
        }
      }, CN_ScrollPanelTimeout);
    }
  }
  // Doesn't work for old version of Chrome 109 and Safari
  // @HostListener('scrollend', ['$event'])
  // public doScrollEndPanel(event:Event):void {
  //   event.preventDefault();
  //   event.stopPropagation();
  //   const elem: Element | null = event.target as Element;
  //   if (ScrollHasMaxUtil.check(elem?.scrollTop, elem?.clientHeight, elem?.scrollHeight)) {
  //     this.requestNextPage.emit();
  //   }
  // }

  constructor(private elementRef: ElementRef) {
  }

  // ** Public API **

  public trackByIdFn(index: number, item: StreamDto): number {
    return item.id;
  }


  public doRedirectToStream(streamId: number): void {
    if (!!streamId) {
      this.redirectToStream.emit(streamId);
    }
  }

  public doActionDuplicate(streamId: number): void {
    if (!!streamId) {
      this.actionDuplicate.emit(streamId);
    }
  }
  public doActionEdit(streamId: number): void {
    if (!!streamId) {
      this.actionEdit.emit(streamId);
    }
  }
  public doActionDelete(streamDto: StreamDto): void {
    if (!!streamDto) {
      this.actionDelete.emit({ id: streamDto.id, title: streamDto.title });
    }
  }

  // ** Private API **

  private scrollToTop(): void {
    if (!!this.elementRef) {
      try {
        this.elementRef.nativeElement.scrollTop = 0;
      } catch (err) { }
    }
  }
}
