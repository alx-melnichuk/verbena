import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PanelStreamInfo } from './panel-stream-info';

describe('PanelStreamInfo', () => {
  let component: PanelStreamInfo;
  let fixture: ComponentFixture<PanelStreamInfo>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [PanelStreamInfo]
    })
    .compileComponents();

    fixture = TestBed.createComponent(PanelStreamInfo);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
