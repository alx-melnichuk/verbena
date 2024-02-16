import { ChangeDetectionStrategy, Component, ElementRef, EventEmitter, HostBinding, HostListener, Input, OnChanges, OnInit, Output, SimpleChanges, ViewEncapsulation } from '@angular/core';
import { CommonModule } from '@angular/common';
import { MatButtonModule } from '@angular/material/button';
import { MatCardModule } from '@angular/material/card';
import { TranslateModule } from '@ngx-translate/core';

import { StreamDto, StreamListDto } from '../stream-api.interface';
import { PIPE_DATE, PIPE_DATE_TIME } from 'src/app/common/constants';
import { PageData, PageDataUtil } from 'src/app/common/page-data';

// InfiniteScrollModule, LogotypeModule
@Component({
  selector: 'app-panel-stream-mini',
  standalone: true,
  imports: [CommonModule, MatButtonModule, MatCardModule, TranslateModule],
  templateUrl: './panel-stream-mini.component.html',
  styleUrls: ['./panel-stream-mini.component.scss'],
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush
})
export class PanelStreamMiniComponent implements OnChanges, OnInit {
  @Input()
  public canDuplicate = false;
  @Input()
  public canEdit = false;
  @Input()
  public canDelete = false;
  @Input()
  public isFuture = false;
  @Input()
  public streamList: StreamListDto = this.emptyStreamListDto();

  @Output()
  readonly requestNextPage: EventEmitter<PageData> = new EventEmitter();
  @Output()
  readonly redirectToStream: EventEmitter<number> = new EventEmitter();
  @Output()
  readonly actionDuplicate: EventEmitter<number> = new EventEmitter();
  @Output()
  readonly actionEdit: EventEmitter<number> = new EventEmitter();
  @Output()
  readonly actionDelete: EventEmitter<StreamDto> = new EventEmitter();

  public streamDTOList: StreamDto[] = [];
  public pageData: PageData = PageDataUtil.create({});
  readonly formatDate = PIPE_DATE;
  readonly formatDateTime = PIPE_DATE_TIME;

  @HostBinding('class.global-scroll')
  public get isGlobalScroll(): boolean { return true; }

  @HostListener('scrollend', ['$event'])
  public doScrollPanel(event:Event):void {
    event.preventDefault();
    event.stopPropagation();
    const element: Element | null = event.target as Element;
    const scrollTopAndHeight = (element?.scrollTop || 0) + (element?.clientHeight || 0);
    const scrollHeight = element?.scrollHeight || 1;
    const result = Math.round((scrollTopAndHeight / scrollHeight)*100)/100;
    if (result == 1) {
      console.log('result == 1');
      this.checkAndGetNextPage(this.pageData);
    }
  }

  constructor(
    private element: ElementRef,
  ) {
  }

  ngOnChanges(changes: SimpleChanges): void {
    if (!!changes['streamList']) {
      this.streamList = !this.streamList ? this.emptyStreamListDto() : this.streamList;
      this.prepareNextStreamsData(this.streamList);
    }
  }

  ngOnInit(): void {
  }

  // ** Public API **

  public trackByIdFn(index: number, item: StreamDto): number {
    return item.id;
  }

  public onScroll(): void {
    console.log(`onScroll();`);
    // 
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
      this.actionDelete.emit(streamDto);
    }
  }

  // ** Private API **

  private checkAndGetNextPage(pageData: PageData): void {
    if (pageData.page < pageData.pages) {
      pageData.page = pageData.page + 1;
      this.requestNextPage.emit(PageDataUtil.create(pageData));
    }
  }

  private prepareNextStreamsData(streamListDto: StreamListDto): void {
    if (1 === streamListDto.page) {
      this.streamDTOList.length = 0;
      this.scrollToTop();
    }
    this.streamDTOList.push(...streamListDto.list);
    this.pageData = PageDataUtil.create(streamListDto);
  }

  private scrollToTop(): void {
    if (!!this.element) {
      try {
        this.element.nativeElement.scrollTop = 0;
      } catch (err) { }
    }
  }

  private emptyStreamListDto(): StreamListDto {
    return { list: [], limit: 0, count: 0, page: 0, pages: -1 };
  }
}
