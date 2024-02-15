import { ChangeDetectionStrategy, ChangeDetectorRef, Component, OnInit, ViewEncapsulation } from '@angular/core';
import { CommonModule } from '@angular/common';
import { SpinnerComponent } from 'src/app/components/spinner/spinner.component';
import { PanelStreamEditorComponent } from '../panel-stream-editor/panel-stream-editor.component';
import { HttpErrorResponse } from '@angular/common/http';
import { ActivatedRoute, Router } from '@angular/router';
import { ROUTE_STREAM } from 'src/app/common/routes';
import { AlertService } from 'src/app/lib-dialog/alert.service';
import { StreamDto, UpdateStreamFileDto } from '../stream-api.interface';
import { StreamService } from '../stream.service';

@Component({
  selector: 'app-stream-edit',
  standalone: true,
  imports: [CommonModule, SpinnerComponent, PanelStreamEditorComponent],
  templateUrl: './stream-edit.component.html',
  styleUrls: ['./stream-edit.component.scss'],
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush
})
export class StreamEditComponent implements OnInit {
  public isLoadDataStream = false;
  public streamDto: StreamDto;
  
  constructor(
    private changeDetectorRef: ChangeDetectorRef,
    private route: ActivatedRoute,
    private router: Router,
    private streamService: StreamService,
    private alertService: AlertService,
  ) {
    console.log(`StreamEditComponent()`); // #-
    this.streamDto = this.route.snapshot.data['streamDto'];
    console.log(`StreamEditComponent() streamDto: `, this.streamDto); // #-
  }
  ngOnInit(): void {
    console.log(`StreamEditComponent().OnInit()`); // #-
  }

  // ** Public API **

  public doUpdateStream(updateStreamFileDto: UpdateStreamFileDto): void {
    this.alertService.hide();
    if (!updateStreamFileDto || (!updateStreamFileDto.createStreamDto && !updateStreamFileDto.modifyStreamDto)) {
      return;
    }
    const isGoToViewStream = true; // ?? (modifyStream.modifyStreamDto.starttime === null);
    const isEdit = (!!updateStreamFileDto.modifyStreamDto);
    const buffPromise: Promise<unknown>[] = [];
    this.isLoadDataStream = true;
    
    if (!!updateStreamFileDto.createStreamDto) {
    //   const addStreamDto = this.streamDTOtoAddStreamDTO(modifyStream.modifyStreamDto);
      buffPromise.push(
        this.streamService.createStream(updateStreamFileDto.createStreamDto, updateStreamFileDto.logoFile));
    } else if (!!updateStreamFileDto.id && !!updateStreamFileDto.modifyStreamDto) {
    //   const updateStreamDTO = this.streamDTOtoUpdateStreamDTO(modifyStream.modifyStreamDto);
      const modifyStreamDto = updateStreamFileDto.modifyStreamDto;
      buffPromise.push(
        this.streamService.modifyStream(updateStreamFileDto.id, modifyStreamDto, updateStreamFileDto.logoFile)
      );
    }
    Promise.all(buffPromise)
      .then((responses) => {
        const streamDto: StreamDto = (responses[0] as StreamDto);
        const goToRoute = (isGoToViewStream ? this.streamService.getLinkForVisitors(streamDto.id, false) : ROUTE_STREAM); 
        // 'ROUTE_STREAM_LIST');
        Promise.resolve()
          .then(() => {
            this.router.navigateByUrl(goToRoute);
          });
      })
      .catch((error: HttpErrorResponse) => {
        console.log(`error: `, error); // #
        const title = (isEdit ? 'stream_edit.error_editing_stream' : 'stream_edit.error_creating_stream');
        // const message = HttpErrorUtil.getMsgs(error)[0];
        const message = 'message';
        this.alertService.showError(message, title);
        throw error;
      })
      .finally(() => {
        this.isLoadDataStream = false;
        this.changeDetectorRef.markForCheck();
      });
  }

  // ** Private API **

  /*private streamDTOtoAddStreamDTO(streamDTO: StreamDTO): AddStreamDTO {
    const result: AddStreamDTO = {
      title: streamDTO.title,
      description: streamDTO.description,
    };
    if (!!streamDTO.starttime) { result.starttime = streamDTO.starttime; }
    if (!!streamDTO.tags) { result.tags = streamDTO.tags; }
    if (!!streamDTO.logo) { result.logoTarget = streamDTO.logo; }
    if (!!streamDTO.source) { result.source = streamDTO.source; }

    return result;
  }*/

  /*private streamDTOtoUpdateStreamDTO(streamDto: StreamDTO): UpdateStreamDTO {
    const result: UpdateStreamDTO = {};
    if (!!streamDto.title) { result.title = streamDto.title; }
    if (!!streamDto.description) { result.description = streamDto.description; }
    if (!!streamDto.starttime) { result.starttime = streamDto.starttime; }
    if (!!streamDto.tags) { result.tags = streamDto.tags; }
    if (!!streamDto.source) { result.source = streamDto.source; }

    return result;
  }*/
}
