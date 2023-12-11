import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PanelLiveStreamsComponent } from './panel-live-streams.component';

describe('PanelLiveStreamsComponent', () => {
  let component: PanelLiveStreamsComponent;
  let fixture: ComponentFixture<PanelLiveStreamsComponent>;

  beforeEach(() => {
    TestBed.configureTestingModule({
      imports: [PanelLiveStreamsComponent]
    });
    fixture = TestBed.createComponent(PanelLiveStreamsComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
