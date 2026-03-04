import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PanelStreamParams } from './panel-stream-params';

describe('PanelStreamParams', () => {
  let component: PanelStreamParams;
  let fixture: ComponentFixture<PanelStreamParams>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [PanelStreamParams]
    })
    .compileComponents();

    fixture = TestBed.createComponent(PanelStreamParams);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
