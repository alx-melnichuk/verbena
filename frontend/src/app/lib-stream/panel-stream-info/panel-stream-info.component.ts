import {
 ChangeDetectionStrategy, Component, ElementRef, EventEmitter, HostBinding, HostListener, Input, Output, ViewEncapsulation
} from '@angular/core';
import { CommonModule } from '@angular/common';
import { MatButtonModule } from '@angular/material/button';
import { MatCardModule } from '@angular/material/card';
import { TranslateModule } from '@ngx-translate/core';

import { PIPE_DATE, PIPE_DATE_TIME } from 'src/app/common/constants';
import { LogotypeComponent } from 'src/app/components/logotype/logotype.component';
import { StreamDto } from '../stream-api.interface';

@Component({
  selector: 'app-panel-stream-info',
  standalone: true,
  imports: [CommonModule, MatButtonModule, MatCardModule, TranslateModule, LogotypeComponent],
  templateUrl: './panel-stream-info.component.html',
  styleUrls: ['./panel-stream-info.component.scss'],
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

  readonly formatDate = PIPE_DATE;
  readonly formatDateTime = PIPE_DATE_TIME;

  @HostBinding('class.global-scroll')
  public get isGlobalScroll(): boolean { return true; }

  @HostListener('scrollend', ['$event'])
  public doScrollPanel(event:Event):void {
    event.preventDefault();
    event.stopPropagation();
    const elem: Element | null = event.target as Element;
    if (this.checkScrollHasMax(elem?.scrollTop, elem?.clientHeight, elem?.scrollHeight)) {
      this.requestNextPage.emit();  
    }
  }

  constructor(
    private element: ElementRef,
  ) {
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
    if (!!this.element) {
      try {
        this.element.nativeElement.scrollTop = 0;
      } catch (err) { }
    }
  }

  private checkScrollHasMax(scrollTop?: number, clientHeight?: number, scrollHeight?: number): boolean {
    const scrollTopAndHeight = (scrollTop || 0) + (clientHeight || 0);
    const result = Math.round((scrollTopAndHeight / (scrollHeight || 1))*100)/100;
    return result > 0.98;
  }
}
