import { ChangeDetectionStrategy, ChangeDetectorRef, Component, OnInit, ViewEncapsulation } from '@angular/core';
import { CommonModule } from '@angular/common';
import { ActivatedRoute, Router } from '@angular/router';

import { AlertService } from 'src/app/lib-dialog/alert.service';
import { StreamService } from 'src/app/lib-stream/stream.service';
import { StreamDto } from 'src/app/lib-stream/stream-api.interface';

import { PanelStreamEditorComponent } from '../panel-stream-editor/panel-stream-editor.component';

@Component({
  selector: 'app-stream-edit',
  standalone: true,
  imports: [CommonModule, PanelStreamEditorComponent],
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

  /*public doModifyStream(modifyStream: ModifyStream): void {
    this.alertService.hide();
    if (!modifyStream || !modifyStream.streamDTO) { return; }
    const isGoToViewStream = (modifyStream.streamDTO.starttime === null);
    const isEdit = (!!modifyStream.streamDTO.id);
    const buffPromise: Promise<unknown>[] = [];
    this.isLoadDataStream = true;
    if (!isEdit) {
      const addStreamDTO = this.streamDTOtoAddStreamDTO(modifyStream.streamDTO);
      buffPromise.push(this.streamService.addStream(addStreamDTO, modifyStream.logoFile));
    } else {
      const updateStreamDTO = this.streamDTOtoUpdateStreamDTO(modifyStream.streamDTO);
      buffPromise.push(
        this.streamService.updateStream(modifyStream.streamDTO.id, updateStreamDTO, modifyStream.logoFile)
      );
    }
    Promise.all(buffPromise)
      .then((responses) => {
        const streamDTO: StreamDTO = (responses[0] as StreamDTO);
        const goToRoute = (isGoToViewStream ? this.streamService.getLinkForVisitors(streamDTO.id, false) : ROUTE_STREAM_LIST);
        Promise.resolve()
          .then(() => {
            this.router.navigateByUrl(goToRoute);
          });
      })
      .catch((error: HttpErrorResponse) => {
        const message = (isEdit ? 'stream_edit.error_editing_stream' : 'stream_edit.error_creating_stream');
        this.alertService.showError(HttpErrorUtil.getMsgs(error)[0], message);
        throw error;
      })
      .finally(() => {
        this.isLoadDataStream = false;
        this.changeDetectorRef.markForCheck();
      });
  }*/

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

  /*private streamDTOtoUpdateStreamDTO(streamDTO: StreamDTO): UpdateStreamDTO {
    const result: UpdateStreamDTO = {};
    if (!!streamDTO.title) { result.title = streamDTO.title; }
    if (!!streamDTO.description) { result.description = streamDTO.description; }
    if (!!streamDTO.starttime) { result.starttime = streamDTO.starttime; }
    if (!!streamDTO.tags) { result.tags = streamDTO.tags; }
    if (!!streamDTO.source) { result.source = streamDTO.source; }

    return result;
  }*/
}
