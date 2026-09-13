import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PanelBrowseParams } from './panel-browse-params';

describe('PanelBrowseParams', () => {
  let component: PanelBrowseParams;
  let fixture: ComponentFixture<PanelBrowseParams>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [PanelBrowseParams]
    })
    .compileComponents();

    fixture = TestBed.createComponent(PanelBrowseParams);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
