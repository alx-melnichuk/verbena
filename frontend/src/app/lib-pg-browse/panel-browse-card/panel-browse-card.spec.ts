import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PanelBrowseCard } from './panel-browse-card';

describe('PanelBrowseCard', () => {
  let component: PanelBrowseCard;
  let fixture: ComponentFixture<PanelBrowseCard>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [PanelBrowseCard]
    })
    .compileComponents();

    fixture = TestBed.createComponent(PanelBrowseCard);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
